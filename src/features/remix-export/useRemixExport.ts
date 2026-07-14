import { useBasicExport } from "./useBasicExport";
import { useRemixGeneration } from "./useRemixGeneration";

type BasicExportOptions = Parameters<typeof useBasicExport>[0];
type RemixGenerationOptions = Parameters<typeof useRemixGeneration>[0];

export function useRemixExport(options: BasicExportOptions & RemixGenerationOptions) {
  return {
    ...useBasicExport(options),
    ...useRemixGeneration(options),
  };
}
