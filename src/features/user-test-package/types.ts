export interface DiagnosticInfo {
  appVersion: string;
  appIdentifier: string;
  generatedAtUnixSeconds: number;
  operatingSystem: string;
  architecture: string;
  executablePath: string;
  dataDirectory: string;
  logsDirectory: string;
  legalDirectory: string;
  tempDirectory: string;
  ffmpegPath: string;
  ffprobePath: string;
  ffmpegAvailable: boolean;
  ffprobeAvailable: boolean;
  bundledFfmpeg: boolean;
  aiConfigured: boolean;
  ttsConfigured: boolean;
  asrConfigured: boolean;
  latestCrashLog: string | null;
}
