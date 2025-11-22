<script lang="ts">
  import { ChevronRight, ChevronDown, Folder, File, Box } from 'lucide-svelte';
  import Hdf5Viewer from './Hdf5Viewer.svelte';

  export let node: any;
  export let level = 0;

  let expanded = level === 0;

  function toggle() {
    if (node.kind === 'group' || (node.kind === 'dataset' && node.children)) {
      expanded = !expanded;
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if ((node.kind === 'group' || (node.kind === 'dataset' && node.children)) && (event.key === 'Enter' || event.key === ' ')) {
      event.preventDefault();
      toggle();
    }
  }

  function isCompoundData(data: any): boolean {
    return Array.isArray(data) && data.length > 0 && typeof data[0] === 'object' && data[0] !== null;
  }

  interface Column {
    key: string;
    type?: string;
  }

  function getColumns(node: any): Column[] {
    if (node.children && node.children.length > 0) {
      return node.children.map((child: any) => ({
        key: child.name,
        type: child.dtype ? child.dtype.replace('Integer(', 'Int(').replace('Float(', 'Float(').replace('Unsigned(', 'UInt(').replace('FixedPoint(', 'Fixed(') : undefined
      }));
    }
    
    if (isCompoundData(node.data_preview)) {
      const keys = new Set<string>();
      node.data_preview.forEach((item: any) => {
        if (typeof item === 'object' && item !== null) {
          Object.keys(item).forEach(key => keys.add(key));
        }
      });
      return Array.from(keys).map(key => ({ key }));
    }
    
    return [];
  }
  
  $: isCompound = isCompoundData(node.data_preview);
  $: columns = isCompound ? getColumns(node) : [];
</script>

<div class="select-none text-sm">
  <div 
    class="flex items-center gap-1 py-1 hover:bg-secondary-bg/50 rounded cursor-pointer transition-colors outline-none focus:ring-1 focus:ring-accent-color"
    style="padding-left: {level * 1.25}rem"
    on:click={toggle}
    on:keydown={handleKeydown}
    role="button"
    tabindex="0"
  >
    {#if node.kind === 'group' || (node.kind === 'dataset' && node.children)}
      <span class="text-secondary-text hover:text-primary-text transition-colors w-4 flex justify-center">
        {#if expanded}
          <ChevronDown size={14} />
        {:else}
          <ChevronRight size={14} />
        {/if}
      </span>
      {#if node.kind === 'group'}
         <Folder size={16} class="text-yellow-500 mr-1" />
      {:else}
         <Box size={16} class="text-blue-400 mr-1" />
      {/if}
    {:else}
      <span class="w-4"></span> 
      {#if node.kind === 'dataset'}
         <Box size={16} class="text-blue-400 mr-1" />
      {:else if node.kind === 'field'}
         <div class="w-4 h-4 mr-1 flex items-center justify-center text-xs text-gray-500">•</div>
      {:else}
         <File size={16} class="text-gray-400 mr-1" />
      {/if}
    {/if}
    
    <span class="font-medium {node.kind === 'group' ? 'text-primary-text' : 'text-secondary-text'}">
      {node.name}
    </span>

    {#if node.kind === 'dataset' || node.kind === 'field'}
      <div class="flex gap-2 ml-3 text-xs opacity-60">
        {#if node.shape && node.shape.length > 0}
           <span class="bg-tertiary-bg px-1 rounded">
             {node.shape.join(' × ')}
           </span>
        {/if}
        {#if node.dtype}
           <span class="text-accent-color">
             {node.dtype.replace('Integer(', 'Int(').replace('Float(', 'Float(').replace('Unsigned(', 'UInt(').replace('FixedPoint(', 'Fixed(')}
           </span>
        {/if}
      </div>
    {/if}

    <!-- Table View for Compound Data -->
    {#if isCompound && expanded}
      <div class="flex-1 ml-4 mt-2 overflow-x-auto">
        <table class="table-auto text-left bg-primary-bg rounded-md border border-border-color text-xs">
          <thead>
            <tr class="bg-secondary-bg">
              {#each columns as col}
                <th class="px-3 py-1.5 font-semibold border-b border-border-color whitespace-nowrap">
                  <div class="flex flex-col">
                    <span>{col.key}</span>
                    {#if col.type}
                      <span class="text-[10px] font-normal text-secondary-text opacity-75">{col.type}</span>
                    {/if}
                  </div>
                </th>
              {/each}
            </tr>
          </thead>
          <tbody>
            {#each node.data_preview as row}
              <tr class="hover:bg-tertiary-bg">
                {#each columns as col}
                  <td class="px-3 py-1 border-b border-border-color/50 whitespace-nowrap font-mono">
                    {row[col.key] !== undefined ? JSON.stringify(row[col.key]) : '-'}
                  </td>
                {/each}
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    <!-- Simple List for Non-Compound Data -->
    {:else if node.data_preview && expanded}
      <div class="flex-1 ml-4 text-xs text-secondary-text">
        <div class="mt-1 bg-primary-bg p-2 rounded-md border border-border-color font-mono">
          {#each node.data_preview as item}
            <div class="truncate">{JSON.stringify(item)}</div>
          {/each}
        </div>
      </div>
    {/if}
  </div>

  <!-- Recursive Children (Hidden if displaying table for Compound Data) -->
  {#if expanded && node.children && !isCompound}
    <div>
      {#each node.children as child}
        <Hdf5Viewer node={child} level={level + 1} />
      {/each}
    </div>
  {/if}
</div>