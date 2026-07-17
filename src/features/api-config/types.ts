export interface ApiConfigStatus {
  platform: "volcengine";
  aiConfigured: boolean;
  aiKeyStored: boolean;
  aiEnvironmentFallback: boolean;
  aiBaseUrl: string;
  aiModel: string;
  ttsConfigured: boolean;
  ttsKeyStored: boolean;
  ttsEnvironmentFallback: boolean;
  ttsResourceId: string;
  ttsSpeaker: string;
}

export interface ApiConfigInput {
  aiApiKey: string;
  aiBaseUrl: string;
  aiModel: string;
  ttsApiKey: string;
  ttsResourceId: string;
  ttsSpeaker: string;
}

export type ApiCredentialKind = "ai" | "tts";
