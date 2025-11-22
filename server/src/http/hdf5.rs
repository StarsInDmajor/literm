use crate::{error::AppError, state::AppState};
use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use hdf5::{File, Group, Dataset};
use hdf5::types::{TypeDescriptor, IntSize, FloatSize, CompoundType};
use serde_json::{json, Value as JsonValue, Map as JsonMap};
use std::io::Cursor;
use std::collections::HashMap;
use byteorder::{LittleEndian, ReadBytesExt};
use hdf5_sys::h5d::H5Dread;
use hdf5_sys::h5s::{H5Screate_simple, H5Sselect_hyperslab, H5Sclose, H5S_SELECT_SET};
use hdf5_sys::h5p::H5P_DEFAULT;

pub fn router() -> Router<AppState> {
    Router::new().route("/api/fs/hdf5/preview", get(preview_handler))
}

#[derive(Debug, Deserialize)]
pub struct Hdf5PreviewQuery {
    pub path: String,
    pub data_limit: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct Hdf5Node {
    pub name: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<Hdf5Node>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shape: Option<Vec<usize>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dtype: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_preview: Option<JsonValue>,
}

#[derive(Debug, Serialize)]
pub struct Hdf5PreviewResponse {
    pub ok: bool,
    pub path: String,
    pub root: Hdf5Node,
}

pub async fn preview_handler(
    State(state): State<AppState>,
    Query(query): Query<Hdf5PreviewQuery>,
) -> Result<Json<Hdf5PreviewResponse>, AppError> {
    if !state.config.features.enable_hdf5 {
        return Err(AppError::Internal("HDF5 support is disabled".into()));
    }

    let resolved = state.fs.resolve_path(&query.path)?;
    let data_limit = query.data_limit.unwrap_or(5);

    let root_node = tokio::task::spawn_blocking(move || {
        let file = File::open(&resolved).map_err(|e| AppError::Internal(format!("Failed to open HDF5 file: {}", e)))?;
        let root_group = file.as_group().map_err(|e| AppError::Internal(format!("Failed to open root group: {}", e)))?;
        
        inspect_group(&root_group, "/", data_limit)
    }).await.map_err(|e| AppError::Internal(e.to_string()))??;

    Ok(Json(Hdf5PreviewResponse {
        ok: true,
        path: query.path,
        root: root_node,
    }))
}

fn format_dtype(desc: &TypeDescriptor) -> String {
    match desc {
        TypeDescriptor::Integer(size) => {
            let bits = match size { IntSize::U1 => 8, IntSize::U2 => 16, IntSize::U4 => 32, IntSize::U8 => 64 };
            format!("Int{}", bits)
        },
        TypeDescriptor::Unsigned(size) => {
             let bits = match size { IntSize::U1 => 8, IntSize::U2 => 16, IntSize::U4 => 32, IntSize::U8 => 64 };
             format!("UInt{}", bits)
        },
        TypeDescriptor::Float(size) => {
             let bits = match size { FloatSize::U4 => 32, FloatSize::U8 => 64 };
             format!("Float{}", bits)
        },
        TypeDescriptor::Boolean => "Bool".to_string(),
        TypeDescriptor::Enum(_) => "Enum".to_string(),
        TypeDescriptor::Compound(_) => "Compound".to_string(),
        TypeDescriptor::FixedArray(ty, dims) => format!("Array{:?} of {}", dims, format_dtype(ty)),
        TypeDescriptor::VarLenArray(ty) => format!("VarArray of {}", format_dtype(ty)),
        TypeDescriptor::FixedAscii(_) | TypeDescriptor::FixedUnicode(_) | TypeDescriptor::VarLenAscii | TypeDescriptor::VarLenUnicode => "String".to_string(),
        _ => format!("{:?}", desc),
    }
}

fn inspect_group(group: &Group, name: &str, data_limit: usize) -> Result<Hdf5Node, AppError> {
    let mut children = Vec::new();
    
    match group.member_names() {
        Ok(members) => {
            for member_name in members {
                if let Ok(dataset) = group.dataset(&member_name) {
                    let shape = dataset.shape();
                    let descriptor = dataset.dtype().and_then(|t| t.to_descriptor()).ok();
                    let dtype_str = descriptor.as_ref().map(format_dtype).unwrap_or_else(|| "unknown".to_string());
                    
                    let mut dataset_children = None;
                    let mut data_preview = None;

                    if let Some(desc) = descriptor.as_ref() {
                        if let TypeDescriptor::Compound(compound_type) = desc {
                            let (rows, columns) = read_compound_data_preview(&dataset, compound_type, data_limit).unwrap_or((None, HashMap::new()));
                            
                            data_preview = rows;

                            // For compound datasets, its 'children' are its fields
                            let fields_nodes = compound_type.fields.iter().map(|f| Hdf5Node {
                                name: f.name.clone(),
                                kind: "field".to_string(),
                                children: None,
                                shape: None,
                                dtype: Some(format_dtype(&f.ty)),
                                data_preview: columns.get(&f.name).cloned(), 
                            }).collect();
                            dataset_children = Some(fields_nodes);
                        } else {
                            data_preview = read_simple_data_preview(&dataset, desc, data_limit).ok();
                        }
                    }

                    children.push(Hdf5Node {
                        name: member_name,
                        kind: "dataset".to_string(),
                        children: dataset_children,
                        shape: Some(shape),
                        dtype: Some(dtype_str),
                        data_preview,
                    });
                } else if let Ok(subgroup) = group.group(&member_name) {
                    children.push(inspect_group(&subgroup, &member_name, data_limit)?);
                } else {
                    children.push(Hdf5Node {
                        name: member_name,
                        kind: "unknown".to_string(),
                        children: None,
                        shape: None,
                        dtype: None,
                        data_preview: None,
                    });
                }
            }
        },
        Err(_) => {} // Ignore if cannot list members
    }

    // Sort children
    children.sort_by(|a, b| {
        let order = |kind: &str| match kind {
            "group" => 0,
            "dataset" => 1,
            "field" => 2,
            _ => 3,
        };
        match order(&a.kind).cmp(&order(&b.kind)) {
            std::cmp::Ordering::Equal => a.name.cmp(&b.name),
            ord => ord,
        }
    });
    
    Ok(Hdf5Node {
        name: name.to_string(),
        kind: "group".to_string(),
        children: Some(children),
        shape: None,
        dtype: None,
        data_preview: None,
    })
}

fn read_simple_data_preview(dataset: &Dataset, desc: &TypeDescriptor, limit: usize) -> Result<JsonValue, AppError> {
    let num_elements = dataset.size();
    let read_count = num_elements.min(limit);

    if read_count == 0 { return Ok(JsonValue::Array(vec![])); }

    // Attempt to read raw bytes for the requested number of elements
    // Note: read_raw() reads ALL elements. This is inefficient for large datasets.
    // Proper optimization requires hyperslab selection which is verbose in high-level crate or needs sys calls.
    // For now, we stick to read_raw() if dataset is small-ish, or risk reading all.
    // Optimization: Use unsafe reader for simple types too if needed, but sticking to safe for now.
    
    // Wait, read_raw<T> checks T size.
    // For simple types, we match T to desc.
    
    match desc {
        TypeDescriptor::Integer(size) => match size {
            IntSize::U1 => read_elements::<i8>(dataset, read_count),
            IntSize::U2 => read_elements::<i16>(dataset, read_count),
            IntSize::U4 => read_elements::<i32>(dataset, read_count),
            IntSize::U8 => read_elements::<i64>(dataset, read_count),
        },
        TypeDescriptor::Unsigned(size) => match size {
            IntSize::U1 => read_elements::<u8>(dataset, read_count),
            IntSize::U2 => read_elements::<u16>(dataset, read_count),
            IntSize::U4 => read_elements::<u32>(dataset, read_count),
            IntSize::U8 => read_elements::<u64>(dataset, read_count),
        },
        TypeDescriptor::Float(size) => match size {
            FloatSize::U4 => read_elements::<f32>(dataset, read_count),
            FloatSize::U8 => read_elements::<f64>(dataset, read_count),
        },
        TypeDescriptor::Boolean => {
             let data = dataset.read_raw::<u8>().map_err(|_| AppError::Internal("Read error".into()))?;
             let vals: Vec<JsonValue> = data.into_iter().take(read_count).map(|b| json!(b != 0)).collect();
             Ok(JsonValue::Array(vals))
        },
        TypeDescriptor::FixedAscii(len) | TypeDescriptor::FixedUnicode(len) => {
             // read_raw<u8> works for char arrays? It treats it as u8 array.
             // Total bytes = total elements * len
             // Dataset size = total elements (rows). 
             // read_raw::<u8> reads N * len bytes.
             let data = dataset.read_raw::<u8>().map_err(|_| AppError::Internal("Read error".into()))?;
             let mut vals = Vec::new();
             for i in 0..read_count {
                 let start = i * len;
                 let end = start + len;
                 if end <= data.len() {
                     let s = String::from_utf8_lossy(&data[start..end]).trim_end_matches('\0').to_string();
                     vals.push(json!(s));
                 }
             }
             Ok(JsonValue::Array(vals))
        },
        _ => Ok(json!("Cannot preview this data type generically")),
    }
}

fn read_elements<T: hdf5::H5Type + Serialize + Copy>(dataset: &Dataset, count: usize) -> Result<JsonValue, AppError> {
    let data = dataset.read_raw::<T>().map_err(|_| AppError::Internal("Read error".into()))?;
    let vals: Vec<JsonValue> = data.into_iter().take(count).map(|v| json!(v)).collect();
    Ok(JsonValue::Array(vals))
}

fn read_compound_data_preview(dataset: &Dataset, compound_type: &CompoundType, limit: usize) -> Result<(Option<JsonValue>, HashMap<String, JsonValue>), AppError> {
    let num_elements = dataset.size();
    let read_count = num_elements.min(limit);

    if read_count == 0 { return Ok((Some(JsonValue::Array(vec![])), HashMap::new())); }

    let record_size = compound_type.size;
    
    // Unsafe read of raw bytes
    let raw_bytes = unsafe {
        let dataset_id = dataset.id();
        // Use the dataset's own type for memory type to ensure matching structure
        let dtype = dataset.dtype().map_err(|e| AppError::Internal(format!("Failed to get dtype: {}", e)))?;
        let mem_type_id = dtype.id();
        
        let mem_dims = [read_count as u64];
        let mem_space_id = H5Screate_simple(1, mem_dims.as_ptr(), std::ptr::null());
        
        let file_space = dataset.space().map_err(|_| AppError::Internal("No space".into()))?;
        let file_space_id = file_space.id();
        let ndim = file_space.ndim();
        
        // Select hyperslab on file space
        // We want to select the first 'read_count' elements along the first dimension (rows)
        // and all elements along other dimensions?
        // Usually HDF5 tables are 1D (N records).
        // If 2D (N x M), we might want N rows, all cols.
        
        let start = vec![0u64; ndim];
        let mut count = vec![1u64; ndim];
        count[0] = read_count as u64; // Select N rows
        
        // For other dimensions, we select the full extent?
        // Actually, H5Sselect_hyperslab count represents the number of blocks.
        // If we just want to slice the first dimension, we need to know the extent of others?
        // dataset.shape() gives us dimensions.
        let shape = dataset.shape();
        for i in 1..ndim {
            count[i] = shape[i] as u64;
        }

        let err = H5Sselect_hyperslab(file_space_id, H5S_SELECT_SET, start.as_ptr(), std::ptr::null(), count.as_ptr(), std::ptr::null());
        if err < 0 {
             H5Sclose(mem_space_id);
             eprintln!("Failed to select hyperslab");
             return Err(AppError::Internal("Failed to select hyperslab".into()));
        }

        // Verify buffer size
        // Total elements = product of counts
        let total_elements: u64 = count.iter().product();
        // But for compound types, 'record_size' is the size of ONE element (which is a struct).
        // So buffer size = total_elements * record_size.
        
        let mut buf = vec![0u8; (total_elements as usize) * record_size];
        let err = H5Dread(dataset_id, mem_type_id, mem_space_id, file_space_id, H5P_DEFAULT, buf.as_mut_ptr() as *mut _);
        
        H5Sclose(mem_space_id);
        
        if err < 0 {
            eprintln!("Failed to read raw data via H5Dread");
            return Err(AppError::Internal("Failed to read raw data".into()));
        }
        buf
    };

    let mut rows = Vec::new();
    let mut columns_map: HashMap<String, Vec<JsonValue>> = HashMap::new();
    
    for field in &compound_type.fields {
        columns_map.insert(field.name.clone(), Vec::new());
    }

    // Parsing raw bytes assumes a packed 1D array of structs.
    // If the dataset was multi-dimensional, the buffer is flattened row-major.
    // We just iterate through it.
    
    // However, our 'read_count' was for the first dimension. 
    // If we have extra dimensions, we are reading read_count * (shape[1]*...) elements.
    // The limit logic in the loop below assumes 1D.
    // Let's handle 1D case primarily for tables.
    
    let actual_elements_read = raw_bytes.len() / record_size;
    
    for i in 0..actual_elements_read {
        let start = i * record_size;
        let end = start + record_size;
        let record_bytes = &raw_bytes[start..end];
        let mut record_map = JsonMap::new();

        for field in &compound_type.fields {
            let field_start = field.offset;
            let field_end = field.offset + field.ty.size();
            
            if field_end <= record_bytes.len() {
                let field_bytes = &record_bytes[field_start..field_end];
                let field_value = read_field_bytes_to_json_value(field_bytes, &field.ty).unwrap_or(json!("error"));
                
                record_map.insert(field.name.clone(), field_value.clone());
                if let Some(col) = columns_map.get_mut(&field.name) {
                    col.push(field_value);
                }
            }
        }
        rows.push(json!(record_map));
    }

    let columns_json: HashMap<String, JsonValue> = columns_map.into_iter()
        .map(|(k, v)| (k, JsonValue::Array(v)))
        .collect();

    Ok((Some(json!(rows)), columns_json))
}

fn read_field_bytes_to_json_value(bytes: &[u8], desc: &TypeDescriptor) -> Result<JsonValue, AppError> {
    let mut cursor = Cursor::new(bytes);
    match desc {
        TypeDescriptor::Integer(size) => match size {
            IntSize::U1 => Ok(json!(cursor.read_i8()?)),
            IntSize::U2 => Ok(json!(cursor.read_i16::<LittleEndian>()?)),
            IntSize::U4 => Ok(json!(cursor.read_i32::<LittleEndian>()?)),
            IntSize::U8 => Ok(json!(cursor.read_i64::<LittleEndian>()?)),
        },
        TypeDescriptor::Unsigned(size) => match size {
            IntSize::U1 => Ok(json!(cursor.read_u8()?)),
            IntSize::U2 => Ok(json!(cursor.read_u16::<LittleEndian>()?)),
            IntSize::U4 => Ok(json!(cursor.read_u32::<LittleEndian>()?)),
            IntSize::U8 => Ok(json!(cursor.read_u64::<LittleEndian>()?)),
        },
        TypeDescriptor::Float(size) => match size {
            FloatSize::U4 => Ok(json!(cursor.read_f32::<LittleEndian>()?)),
            FloatSize::U8 => Ok(json!(cursor.read_f64::<LittleEndian>()?)),
        },
        TypeDescriptor::Boolean => Ok(json!(cursor.read_u8()? != 0)),
        TypeDescriptor::FixedAscii(_) | TypeDescriptor::FixedUnicode(_) => {
            Ok(json!(String::from_utf8_lossy(bytes).trim_end_matches('\0')))
        },
        _ => Ok(json!("Unsupported")), 
    }
}