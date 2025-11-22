// This is a sample JavaScript file for testing text preview
function calculateSum(numbers) {
  return numbers.reduce((sum, num) => sum + num, 0);
}

function findMaximum(numbers) {
  return Math.max(...numbers);
}

// Example usage
const data = [1, 2, 3, 4, 5, 10, 15, 20];
const total = calculateSum(data);
const maximum = findMaximum(data);

console.log('Total:', total);
console.log('Maximum:', maximum);

// Array methods example
const doubled = data.map(x => x * 2);
const evens = data.filter(x => x % 2 === 0);
const hasLargeNumbers = data.some(x => x > 100);

console.log('Doubled:', doubled);
console.log('Even numbers:', evens);
console.log('Has large numbers:', hasLargeNumbers);