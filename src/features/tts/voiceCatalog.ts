export type TtsVoiceLanguage = "zh" | "en" | "ja" | "es" | "id";
export type TtsVoiceScene = "general" | "narration" | "commercial" | "character" | "youth";
export type TtsVoiceGender = "female" | "male";
export type TtsVoiceTone = "mint" | "blue" | "amber" | "rose" | "violet" | "slate";

export interface TtsVoiceCatalogItem {
  id: string;
  name: string;
  gender: TtsVoiceGender;
  languages: TtsVoiceLanguage[];
  scenes: TtsVoiceScene[];
  tone: TtsVoiceTone;
  description: string;
  previewUrl: string;
  recommended?: boolean;
}

const PREVIEW_ROOT =
  "https://lf3-static.bytednsdoc.com/obj/eden-cn/lm_hz_ihsph/ljhwZthlaukjlkulzlp/portal/bigtts";

function preview(fileName: string) {
  return `${PREVIEW_ROOT}/${fileName}`;
}

export const TTS_LANGUAGE_LABELS: Record<TtsVoiceLanguage, string> = {
  zh: "中文",
  en: "英语",
  ja: "日语",
  es: "西班牙语",
  id: "印尼语",
};

export const TTS_SCENE_LABELS: Record<TtsVoiceScene, string> = {
  general: "通用",
  narration: "口播解说",
  commercial: "商品营销",
  character: "角色演绎",
  youth: "年轻活力",
};

export const TTS_GENDER_LABELS: Record<TtsVoiceGender, string> = {
  female: "女声",
  male: "男声",
};

export const TTS_VOICE_CATALOG: TtsVoiceCatalogItem[] = [
  {
    id: "zh_female_vv_uranus_bigtts",
    name: "Vivi 2.0",
    gender: "female",
    languages: ["zh", "ja", "id", "es"],
    scenes: ["general", "narration"],
    tone: "violet",
    description: "自然清晰、适应面广，多语种内容优先推荐。",
    previewUrl: preview("zh_female_vv_uranus_bigtts.wav"),
    recommended: true,
  },
  {
    id: "zh_female_xiaohe_uranus_bigtts",
    name: "小何 2.0",
    gender: "female",
    languages: ["zh", "en"],
    scenes: ["general", "commercial"],
    tone: "mint",
    description: "亲切自然，适合日常口播、好物分享和教程。",
    previewUrl: preview("zh_female_xiaohe_uranus_bigtts.mp3"),
    recommended: true,
  },
  {
    id: "zh_male_m191_uranus_bigtts",
    name: "云舟 2.0",
    gender: "male",
    languages: ["zh", "en"],
    scenes: ["general", "narration"],
    tone: "blue",
    description: "沉稳耐听，适合知识讲解、产品介绍和纪录片旁白。",
    previewUrl: preview("zh_male_m191_uranus_bigtts.mp3"),
    recommended: true,
  },
  {
    id: "zh_male_taocheng_uranus_bigtts",
    name: "小天 2.0",
    gender: "male",
    languages: ["zh", "en"],
    scenes: ["commercial", "youth"],
    tone: "amber",
    description: "年轻明快，适合数码、生活方式和节奏感口播。",
    previewUrl: preview("zh_male_taocheng_uranus_bigtts.mp3"),
    recommended: true,
  },
  {
    id: "zh_male_liufei_uranus_bigtts",
    name: "刘飞 2.0",
    gender: "male",
    languages: ["zh", "en"],
    scenes: ["narration", "commercial"],
    tone: "slate",
    description: "成熟利落，适合商业解说、测评和观点内容。",
    previewUrl: preview("zh_male_liufei_uranus_bigtts.mp3"),
  },
  {
    id: "zh_female_sophie_uranus_bigtts",
    name: "魅力苏菲",
    gender: "female",
    languages: ["zh", "en"],
    scenes: ["commercial", "character"],
    tone: "rose",
    description: "成熟有质感，适合美妆、时尚和品牌内容。",
    previewUrl: preview("zh_male_sophie_uranus_bigtts.mp3"),
  },
  {
    id: "zh_female_qingxinnvsheng_uranus_bigtts",
    name: "清新女声",
    gender: "female",
    languages: ["zh", "en"],
    scenes: ["general", "narration"],
    tone: "mint",
    description: "轻柔清爽，适合旅行、美食、家居和治愈内容。",
    previewUrl: preview("zh_female_qingxinnvsheng_uranus_bigtts.mp3"),
  },
  {
    id: "zh_female_cancan_uranus_bigtts",
    name: "知性灿灿",
    gender: "female",
    languages: ["zh", "en"],
    scenes: ["narration", "commercial"],
    tone: "blue",
    description: "知性克制，适合课程、职场和专业产品介绍。",
    previewUrl: preview("zh_female_cancan_uranus_bigtts.mp3"),
  },
  {
    id: "zh_female_sajiaoxuemei_uranus_bigtts",
    name: "灵动学妹",
    gender: "female",
    languages: ["zh", "en"],
    scenes: ["character", "youth"],
    tone: "rose",
    description: "活泼俏皮，适合校园、剧情和年轻化内容。",
    previewUrl: preview("zh_female_sajiaoxuemei_uranus_bigtts.mp3"),
  },
  {
    id: "zh_female_tianmeixiaoyuan_uranus_bigtts",
    name: "甜美小源",
    gender: "female",
    languages: ["zh", "en"],
    scenes: ["commercial", "youth"],
    tone: "rose",
    description: "甜美明亮，适合美妆、零食、穿搭和种草。",
    previewUrl: preview("zh_female_tianmeixiaoyuan_uranus_bigtts.mp3"),
  },
  {
    id: "zh_female_tianmeitaozi_uranus_bigtts",
    name: "甜美桃子",
    gender: "female",
    languages: ["zh", "en"],
    scenes: ["character", "youth"],
    tone: "amber",
    description: "元气甜润，适合轻剧情、萌宠和生活分享。",
    previewUrl: preview("zh_female_tianmeitaozi_uranus_bigtts.mp3"),
  },
  {
    id: "zh_female_shuangkuaisisi_uranus_bigtts",
    name: "爽快思思",
    gender: "female",
    languages: ["zh", "en"],
    scenes: ["commercial", "youth"],
    tone: "amber",
    description: "节奏爽快，适合促销、探店和高能短视频。",
    previewUrl: preview("zh_female_shuangkuaisisi_uranus_bigtts.mp3"),
  },
  {
    id: "zh_female_linjianvhai_uranus_bigtts",
    name: "邻家女孩",
    gender: "female",
    languages: ["zh", "en"],
    scenes: ["general", "youth"],
    tone: "mint",
    description: "亲和松弛，适合日常记录、分享和陪伴式口播。",
    previewUrl: preview("zh_female_linjianvhai_uranus_bigtts.mp3"),
  },
  {
    id: "zh_male_shaonianzixin_uranus_bigtts",
    name: "少年梓辛",
    gender: "male",
    languages: ["zh", "en"],
    scenes: ["character", "youth"],
    tone: "blue",
    description: "少年感清晰，适合游戏、动漫和青春剧情。",
    previewUrl: preview("zh_male_shaonianzixin_uranus_bigtts.mp3"),
  },
  {
    id: "zh_female_meilinvyou_uranus_bigtts",
    name: "魅力女友",
    gender: "female",
    languages: ["zh", "en"],
    scenes: ["character", "commercial"],
    tone: "violet",
    description: "温柔有表现力，适合情感、剧情和品牌口播。",
    previewUrl: preview("zh_female_meilinvyou_uranus_bigtts.mp3"),
  },
  {
    id: "en_male_tim_uranus_bigtts",
    name: "Tim",
    gender: "male",
    languages: ["en"],
    scenes: ["general", "narration"],
    tone: "slate",
    description: "自然英语男声，适合讲解、产品演示和国际内容。",
    previewUrl: preview("en_male_tim_uranus_bigtts.mp3"),
    recommended: true,
  },
  {
    id: "en_female_dacey_uranus_bigtts",
    name: "Dacey",
    gender: "female",
    languages: ["en"],
    scenes: ["general", "commercial"],
    tone: "rose",
    description: "清晰自然的英语女声，适合品牌、教程和生活内容。",
    previewUrl: preview("en_female_dacey_uranus_bigtts.mp3"),
    recommended: true,
  },
  {
    id: "en_female_stokie_uranus_bigtts",
    name: "Stokie",
    gender: "female",
    languages: ["en"],
    scenes: ["narration", "character"],
    tone: "violet",
    description: "富有表达力的英语女声，适合故事、播客和剧情。",
    previewUrl: preview("en_female_stokie_uranus_bigtts.mp3"),
  },
  {
    id: "zh_male_wennuanahu_uranus_bigtts",
    name: "温暖阿虎",
    gender: "male",
    languages: ["zh", "en"],
    scenes: ["general", "narration"],
    tone: "amber",
    description: "温暖可靠，适合生活感悟、家居和知识分享。",
    previewUrl: preview("zh_male_wennuanahu_uranus_bigtts.mp3"),
  },
  {
    id: "zh_male_naiqimengwa_uranus_bigtts",
    name: "奶气萌娃",
    gender: "male",
    languages: ["zh", "en"],
    scenes: ["character", "youth"],
    tone: "mint",
    description: "童真可爱，适合亲子、萌宠和轻松角色内容。",
    previewUrl: preview("zh_male_naiqimengwa_uranus_bigtts.mp3"),
  },
  {
    id: "zh_female_popo_uranus_bigtts",
    name: "慈祥长辈",
    gender: "female",
    languages: ["zh", "en"],
    scenes: ["character", "narration"],
    tone: "slate",
    description: "温和沉稳，适合家庭故事、传统文化和叙事内容。",
    previewUrl: preview("zh_female_popo_uranus_bigtts.mp3"),
  },
  {
    id: "zh_female_kailangjiejie_uranus_bigtts",
    name: "开朗姐姐",
    gender: "female",
    languages: ["zh", "en"],
    scenes: ["commercial", "youth"],
    tone: "amber",
    description: "大方有感染力，适合探店、直播切片和好物推荐。",
    previewUrl: preview("zh_female_kailangjiejie_uranus_bigtts.mp3"),
  },
  {
    id: "zh_male_baqiqingshu_uranus_bigtts",
    name: "沉稳青叔",
    gender: "male",
    languages: ["zh", "en"],
    scenes: ["narration", "commercial"],
    tone: "slate",
    description: "低沉有力量，适合汽车、商业、历史和硬核测评。",
    previewUrl: preview("zh_male_baqiqingshu_uranus_bigtts.mp3"),
  },
  {
    id: "zh_male_xuanyijieshuo_uranus_bigtts",
    name: "悬疑解说",
    gender: "male",
    languages: ["zh", "en"],
    scenes: ["narration", "character"],
    tone: "violet",
    description: "氛围感强，适合悬疑故事、影视解说和反转内容。",
    previewUrl: preview("zh_male_xuanyijieshuo_uranus_bigtts.mp3"),
  },
  {
    id: "zh_female_gufengshaoyu_uranus_bigtts",
    name: "古风少御",
    gender: "female",
    languages: ["zh", "en"],
    scenes: ["character", "narration"],
    tone: "violet",
    description: "古典柔和，适合国风、文化、小说和剧情旁白。",
    previewUrl: preview("zh_female_gufengshaoyu_uranus_bigtts.mp3"),
  },
];

export function findTtsVoice(speakerId: string) {
  return TTS_VOICE_CATALOG.find((voice) => voice.id === speakerId) ?? null;
}
