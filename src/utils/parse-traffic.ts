const UNITS = ["B", "KB", "MB", "GB", "TB"];

const parseTraffic = (value: number): [string, string] => {
  if (value <= 0) {
    return ["0", "B"];
  }

  let unitIndex = 0;
  let processedValue = value;

  while (processedValue >= 1024 && unitIndex < UNITS.length - 1) {
    processedValue /= 1024;
    unitIndex++;
  }

  const formattedValue = processedValue < 10 
    ? processedValue.toFixed(1) 
    : Math.round(processedValue).toString();

  return [formattedValue, UNITS[unitIndex]];
};

export default parseTraffic;
