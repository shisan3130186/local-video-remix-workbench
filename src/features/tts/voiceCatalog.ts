export type TtsVoiceLanguage = "zh" | "en" | "ja" | "es" | "id";
export type TtsVoiceScene = "general" | "narration" | "commercial" | "character" | "youth" | "customer-service" | "education" | "audiobook" | "multilingual";
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
  avatarUrl: string;
  recommended?: boolean;
}

const PREVIEW_ROOT =
  "https://lf3-static.bytednsdoc.com/obj/eden-cn/lm_hz_ihsph/ljhwZthlaukjlkulzlp/portal/bigtts";

function preview(fileName: string) {
  return `${PREVIEW_ROOT}/${fileName}`;
}
function mp3ById(speakerId: string) {
  return preview(`${speakerId}.mp3`);
}

function avatarByName(voiceName: string) {
  return `${PREVIEW_ROOT}/avatar/${encodeURIComponent(voiceName)}.png`;
}

const LOCAL_AVATARS = new Set<string>([
  "ICL_uranus_zh_female_chuliannvyou_tob",
  "ICL_uranus_zh_female_chunchenvsheng_tob",
  "ICL_uranus_zh_female_guaiqiaokeer_tob",
  "ICL_uranus_zh_female_kailangtingting_tob",
  "ICL_uranus_zh_female_kaixinxiaohong_tob",
  "ICL_uranus_zh_female_kefuwanjun_tob",
  "ICL_uranus_zh_female_lingdongxinxin_tob",
  "ICL_uranus_zh_female_lixingyuanzi_tob",
  "ICL_uranus_zh_female_nuanxinqianqian_tob",
  "ICL_uranus_zh_female_qingtianmeimei_tob",
  "ICL_uranus_zh_female_qingtiantaotao_tob",
  "ICL_uranus_zh_female_qingxinshaonv_tob",
  "ICL_uranus_zh_female_qingxixiaoxue_tob",
  "ICL_uranus_zh_female_ruanmengtangtang_tob",
  "ICL_uranus_zh_female_ruanmengtuanzi_tob",
  "ICL_uranus_zh_female_tianmeitaozi_tob",
  "ICL_uranus_zh_female_tianmeixiaoju_tob",
  "ICL_uranus_zh_female_tianmeixiaoyu_tob",
  "ICL_uranus_zh_female_tiexinguimi_tob",
  "ICL_uranus_zh_female_tiexinmeimei_tob",
  "ICL_uranus_zh_female_wenroubaiyueguang_tob",
  "ICL_uranus_zh_female_wenrounvshen_tob",
  "ICL_uranus_zh_female_xiuliqianqian_tob",
  "ICL_uranus_zh_female_yuanqitianmei_tob",
  "ICL_uranus_zh_female_zhixingwenwan_tob",
  "ICL_uranus_zh_female_zhixinjiejie_tob",
  "ICL_uranus_zh_male_chunhoudiyin_tob",
  "ICL_uranus_zh_male_diyinchenyu_tob",
  "ICL_uranus_zh_male_paoxiaoxiaoge_tob",
  "ICL_uranus_zh_male_yangyang_tob",
  "zh_female_cancan_uranus_bigtts",
  "zh_female_gufengshaoyu_uranus_bigtts",
  "zh_female_kailangjiejie_uranus_bigtts",
  "zh_female_linjianvhai_uranus_bigtts",
  "zh_female_meilinvyou_uranus_bigtts",
  "zh_female_popo_uranus_bigtts",
  "zh_female_qingxinnvsheng_uranus_bigtts",
  "zh_female_sajiaoxuemei_uranus_bigtts",
  "zh_female_shuangkuaisisi_uranus_bigtts",
  "zh_female_sophie_uranus_bigtts",
  "zh_female_tianmeitaozi_uranus_bigtts",
  "zh_female_tianmeixiaoyuan_uranus_bigtts",
  "zh_female_vv_uranus_bigtts",
  "zh_female_xiaohe_uranus_bigtts",
  "zh_male_baqiqingshu_uranus_bigtts",
  "zh_male_chenwenmingzai_tob",
  "zh_male_liufei_uranus_bigtts",
  "zh_male_m191_uranus_bigtts",
  "zh_male_naiqimengwa_uranus_bigtts",
  "zh_male_shaonianzixin_uranus_bigtts",
  "zh_male_taocheng_uranus_bigtts",
  "zh_male_wennuanahu_uranus_bigtts",
  "zh_male_xuanyijieshuo_uranus_bigtts",
]);

function resolveAvatarUrl(voiceId: string, voiceName: string) {
  if (LOCAL_AVATARS.has(voiceId)) {
    return `/voice-avatars/${voiceId}.png`;
  }
  return avatarByName(voiceName);
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
  "customer-service": "客服场景",
  education: "教育场景",
  audiobook: "有声阅读",
  multilingual: "多语种",
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

    avatarUrl: resolveAvatarUrl("zh_female_vv_uranus_bigtts", "Vivi 2.0"),    recommended: true,
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

    avatarUrl: resolveAvatarUrl("zh_female_xiaohe_uranus_bigtts", "小何 2.0"),    recommended: true,
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

    avatarUrl: resolveAvatarUrl("zh_male_m191_uranus_bigtts", "云舟 2.0"),    recommended: true,
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

    avatarUrl: resolveAvatarUrl("zh_male_taocheng_uranus_bigtts", "小天 2.0"),    recommended: true,
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

    avatarUrl: resolveAvatarUrl("zh_male_liufei_uranus_bigtts", "刘飞 2.0"),  },
  {
    id: "zh_female_sophie_uranus_bigtts",
    name: "魅力苏菲",
    gender: "female",
    languages: ["zh", "en"],
    scenes: ["commercial", "character"],
    tone: "rose",
    description: "成熟有质感，适合美妆、时尚和品牌内容。",
    previewUrl: preview("zh_male_sophie_uranus_bigtts.mp3"),

    avatarUrl: resolveAvatarUrl("zh_female_sophie_uranus_bigtts", "魅力苏菲"),  },
  {
    id: "zh_female_qingxinnvsheng_uranus_bigtts",
    name: "清新女声",
    gender: "female",
    languages: ["zh", "en"],
    scenes: ["general", "narration"],
    tone: "mint",
    description: "轻柔清爽，适合旅行、美食、家居和治愈内容。",
    previewUrl: preview("zh_female_qingxinnvsheng_uranus_bigtts.mp3"),

    avatarUrl: resolveAvatarUrl("zh_female_qingxinnvsheng_uranus_bigtts", "清新女声"),  },
  {
    id: "zh_female_cancan_uranus_bigtts",
    name: "知性灿灿",
    gender: "female",
    languages: ["zh", "en"],
    scenes: ["narration", "commercial"],
    tone: "blue",
    description: "知性克制，适合课程、职场和专业产品介绍。",
    previewUrl: preview("zh_female_cancan_uranus_bigtts.mp3"),

    avatarUrl: resolveAvatarUrl("zh_female_cancan_uranus_bigtts", "知性灿灿"),  },
  {
    id: "zh_female_sajiaoxuemei_uranus_bigtts",
    name: "灵动学妹",
    gender: "female",
    languages: ["zh", "en"],
    scenes: ["character", "youth"],
    tone: "rose",
    description: "活泼俏皮，适合校园、剧情和年轻化内容。",
    previewUrl: preview("zh_female_sajiaoxuemei_uranus_bigtts.mp3"),

    avatarUrl: resolveAvatarUrl("zh_female_sajiaoxuemei_uranus_bigtts", "灵动学妹"),  },
  {
    id: "zh_female_tianmeixiaoyuan_uranus_bigtts",
    name: "甜美小源",
    gender: "female",
    languages: ["zh", "en"],
    scenes: ["commercial", "youth"],
    tone: "rose",
    description: "甜美明亮，适合美妆、零食、穿搭和种草。",
    previewUrl: preview("zh_female_tianmeixiaoyuan_uranus_bigtts.mp3"),

    avatarUrl: resolveAvatarUrl("zh_female_tianmeixiaoyuan_uranus_bigtts", "甜美小源"),  },
  {
    id: "zh_female_tianmeitaozi_uranus_bigtts",
    name: "甜美桃子",
    gender: "female",
    languages: ["zh", "en"],
    scenes: ["character", "youth"],
    tone: "amber",
    description: "元气甜润，适合轻剧情、萌宠和生活分享。",
    previewUrl: preview("zh_female_tianmeitaozi_uranus_bigtts.mp3"),

    avatarUrl: resolveAvatarUrl("zh_female_tianmeitaozi_uranus_bigtts", "甜美桃子"),  },
  {
    id: "zh_female_shuangkuaisisi_uranus_bigtts",
    name: "爽快思思",
    gender: "female",
    languages: ["zh", "en"],
    scenes: ["commercial", "youth"],
    tone: "amber",
    description: "节奏爽快，适合促销、探店和高能短视频。",
    previewUrl: preview("zh_female_shuangkuaisisi_uranus_bigtts.mp3"),

    avatarUrl: resolveAvatarUrl("zh_female_shuangkuaisisi_uranus_bigtts", "爽快思思"),  },
  {
    id: "zh_female_linjianvhai_uranus_bigtts",
    name: "邻家女孩",
    gender: "female",
    languages: ["zh", "en"],
    scenes: ["general", "youth"],
    tone: "mint",
    description: "亲和松弛，适合日常记录、分享和陪伴式口播。",
    previewUrl: preview("zh_female_linjianvhai_uranus_bigtts.mp3"),

    avatarUrl: resolveAvatarUrl("zh_female_linjianvhai_uranus_bigtts", "邻家女孩"),  },
  {
    id: "zh_male_shaonianzixin_uranus_bigtts",
    name: "少年梓辛",
    gender: "male",
    languages: ["zh", "en"],
    scenes: ["character", "youth"],
    tone: "blue",
    description: "少年感清晰，适合游戏、动漫和青春剧情。",
    previewUrl: preview("zh_male_shaonianzixin_uranus_bigtts.mp3"),

    avatarUrl: resolveAvatarUrl("zh_male_shaonianzixin_uranus_bigtts", "少年梓辛"),  },
  {
    id: "zh_female_meilinvyou_uranus_bigtts",
    name: "魅力女友",
    gender: "female",
    languages: ["zh", "en"],
    scenes: ["character", "commercial"],
    tone: "violet",
    description: "温柔有表现力，适合情感、剧情和品牌口播。",
    previewUrl: preview("zh_female_meilinvyou_uranus_bigtts.mp3"),

    avatarUrl: resolveAvatarUrl("zh_female_meilinvyou_uranus_bigtts", "魅力女友"),  },
  {
    id: "zh_male_wennuanahu_uranus_bigtts",
    name: "温暖阿虎",
    gender: "male",
    languages: ["zh", "en"],
    scenes: ["general", "narration"],
    tone: "amber",
    description: "温暖可靠，适合生活感悟、家居和知识分享。",
    previewUrl: preview("zh_male_wennuanahu_uranus_bigtts.mp3"),

    avatarUrl: resolveAvatarUrl("zh_male_wennuanahu_uranus_bigtts", "温暖阿虎"),  },
  {
    id: "zh_male_naiqimengwa_uranus_bigtts",
    name: "奶气萌娃",
    gender: "male",
    languages: ["zh", "en"],
    scenes: ["character", "youth"],
    tone: "mint",
    description: "童真可爱，适合亲子、萌宠和轻松角色内容。",
    previewUrl: preview("zh_male_naiqimengwa_uranus_bigtts.mp3"),

    avatarUrl: resolveAvatarUrl("zh_male_naiqimengwa_uranus_bigtts", "奶气萌娃"),  },
  {
    id: "zh_female_popo_uranus_bigtts",
    name: "慈祥长辈",
    gender: "female",
    languages: ["zh", "en"],
    scenes: ["character", "narration"],
    tone: "slate",
    description: "温和沉稳，适合家庭故事、传统文化和叙事内容。",
    previewUrl: preview("zh_female_popo_uranus_bigtts.mp3"),

    avatarUrl: resolveAvatarUrl("zh_female_popo_uranus_bigtts", "慈祥长辈"),  },
  {
    id: "zh_female_kailangjiejie_uranus_bigtts",
    name: "开朗姐姐",
    gender: "female",
    languages: ["zh", "en"],
    scenes: ["commercial", "youth"],
    tone: "amber",
    description: "大方有感染力，适合探店、直播切片和好物推荐。",
    previewUrl: preview("zh_female_kailangjiejie_uranus_bigtts.mp3"),

    avatarUrl: resolveAvatarUrl("zh_female_kailangjiejie_uranus_bigtts", "开朗姐姐"),  },
  {
    id: "zh_male_baqiqingshu_uranus_bigtts",
    name: "沉稳青叔",
    gender: "male",
    languages: ["zh", "en"],
    scenes: ["narration", "commercial"],
    tone: "slate",
    description: "低沉有力量，适合汽车、商业、历史和硬核测评。",
    previewUrl: preview("zh_male_baqiqingshu_uranus_bigtts.mp3"),

    avatarUrl: resolveAvatarUrl("zh_male_baqiqingshu_uranus_bigtts", "沉稳青叔"),  },
  {
    id: "zh_male_xuanyijieshuo_uranus_bigtts",
    name: "悬疑解说",
    gender: "male",
    languages: ["zh", "en"],
    scenes: ["narration", "character"],
    tone: "violet",
    description: "氛围感强，适合悬疑故事、影视解说和反转内容。",
    previewUrl: preview("zh_male_xuanyijieshuo_uranus_bigtts.mp3"),

    avatarUrl: resolveAvatarUrl("zh_male_xuanyijieshuo_uranus_bigtts", "悬疑解说"),  },
  {
    id: "zh_female_gufengshaoyu_uranus_bigtts",
    name: "古风少御",
    gender: "female",
    languages: ["zh", "en"],
    scenes: ["character", "narration"],
    tone: "violet",
    description: "古典柔和，适合国风、文化、小说和剧情旁白。",
    previewUrl: preview("zh_female_gufengshaoyu_uranus_bigtts.mp3"),

    avatarUrl: resolveAvatarUrl("zh_female_gufengshaoyu_uranus_bigtts", "古风少御"),  },
  { id: "ICL_uranus_zh_female_kefuwanjun_tob", name: "客服婉君 2.0", gender: "female", languages: ["zh"], scenes: ["customer-service"], tone: "amber", description: "客服、专业、低调节奏。", previewUrl: mp3ById("ICL_uranus_zh_female_kefuwanjun_tob"), avatarUrl: resolveAvatarUrl("ICL_uranus_zh_female_kefuwanjun_tob", "客服婉君 2.0") },
  { id: "ICL_uranus_zh_female_guaiqiaokeer_tob", name: "乖巧可儿 2.0", gender: "female", languages: ["zh"], scenes: ["customer-service"], tone: "mint", description: "简洁亲和，客户友好。", previewUrl: mp3ById("ICL_uranus_zh_female_guaiqiaokeer_tob"), avatarUrl: resolveAvatarUrl("ICL_uranus_zh_female_guaiqiaokeer_tob", "乖巧可儿 2.0") },
  { id: "ICL_uranus_zh_female_kailangtingting_tob", name: "开朗婷婷 2.0", gender: "female", languages: ["zh"], scenes: ["customer-service"], tone: "amber", description: "开朗、营销、首呼客服。", previewUrl: mp3ById("ICL_uranus_zh_female_kailangtingting_tob"), avatarUrl: resolveAvatarUrl("ICL_uranus_zh_female_kailangtingting_tob", "开朗婷婷 2.0") },
  { id: "ICL_uranus_zh_female_lixingyuanzi_tob", name: "理性圆子 2.0", gender: "female", languages: ["zh"], scenes: ["customer-service"], tone: "blue", description: "金融、技术、咨询客服。", previewUrl: mp3ById("ICL_uranus_zh_female_lixingyuanzi_tob"), avatarUrl: resolveAvatarUrl("ICL_uranus_zh_female_lixingyuanzi_tob", "理性圆子 2.0") },
  { id: "ICL_uranus_zh_female_qingxixiaoxue_tob", name: "清晰小雪 2.0", gender: "female", languages: ["zh"], scenes: ["customer-service"], tone: "mint", description: "售后、咨询、回访。", previewUrl: mp3ById("ICL_uranus_zh_female_qingxixiaoxue_tob"), avatarUrl: resolveAvatarUrl("ICL_uranus_zh_female_qingxixiaoxue_tob", "清晰小雪 2.0") },
  { id: "ICL_uranus_zh_female_qingtiantaotao_tob", name: "清甜桃桃 2.0", gender: "female", languages: ["zh"], scenes: ["customer-service"], tone: "rose", description: "甜感亲切、客服。", previewUrl: mp3ById("ICL_uranus_zh_female_qingtiantaotao_tob"), avatarUrl: resolveAvatarUrl("ICL_uranus_zh_female_qingtiantaotao_tob", "清甜桃桃 2.0") },
  { id: "ICL_uranus_zh_female_qingtianmeimei_tob", name: "清甜莓莓 2.0", gender: "female", languages: ["zh"], scenes: ["customer-service"], tone: "rose", description: "甜美耐心、客服。", previewUrl: mp3ById("ICL_uranus_zh_female_qingtianmeimei_tob"), avatarUrl: resolveAvatarUrl("ICL_uranus_zh_female_qingtianmeimei_tob", "清甜莓莓 2.0") },
  { id: "ICL_uranus_zh_male_chenwenmingzai_tob", name: "沉稳明仔 2.0", gender: "male", languages: ["zh"], scenes: ["customer-service"], tone: "blue", description: "专业、客服。", previewUrl: mp3ById("ICL_uranus_zh_male_chenwenmingzai_tob"), avatarUrl: resolveAvatarUrl("ICL_uranus_zh_male_chenwenmingzai_tob", "沉稳明仔 2.0") },
  { id: "ICL_uranus_zh_female_tianmeixiaoyu_tob", name: "甜美小雨 2.0", gender: "female", languages: ["zh"], scenes: ["customer-service"], tone: "rose", description: "甜美、客服。", previewUrl: mp3ById("ICL_uranus_zh_female_tianmeixiaoyu_tob"), avatarUrl: resolveAvatarUrl("ICL_uranus_zh_female_tianmeixiaoyu_tob", "甜美小雨 2.0") },
  { id: "ICL_uranus_zh_female_tianmeixiaoju_tob", name: "甜美小橘 2.0", gender: "female", languages: ["zh"], scenes: ["customer-service"], tone: "rose", description: "甜感、客服。", previewUrl: mp3ById("ICL_uranus_zh_female_tianmeixiaoju_tob"), avatarUrl: resolveAvatarUrl("ICL_uranus_zh_female_tianmeixiaoju_tob", "甜美小橘 2.0") },
  { id: "ICL_uranus_zh_female_xiuliqianqian_tob", name: "秀丽倩倩 2.0", gender: "female", languages: ["zh"], scenes: ["customer-service"], tone: "amber", description: "友好、客服。", previewUrl: mp3ById("ICL_uranus_zh_female_xiuliqianqian_tob"), avatarUrl: resolveAvatarUrl("ICL_uranus_zh_female_xiuliqianqian_tob", "秀丽倩倩 2.0") },
  { id: "ICL_uranus_zh_female_lingdongxinxin_tob", name: "灵动欣欣 2.0", gender: "female", languages: ["zh"], scenes: ["customer-service"], tone: "amber", description: "灵动发现、客服。", previewUrl: mp3ById("ICL_uranus_zh_female_lingdongxinxin_tob"), avatarUrl: resolveAvatarUrl("ICL_uranus_zh_female_lingdongxinxin_tob", "灵动欣欣 2.0") },
  { id: "ICL_uranus_zh_female_ruanmengtangtang_tob", name: "软萌糖糖 2.0", gender: "female", languages: ["zh"], scenes: ["customer-service"], tone: "rose", description: "软萌、客服。", previewUrl: mp3ById("ICL_uranus_zh_female_ruanmengtangtang_tob"), avatarUrl: resolveAvatarUrl("ICL_uranus_zh_female_ruanmengtangtang_tob", "软萌糖糖 2.0") },
  { id: "ICL_uranus_zh_female_ruanmengtuanzi_tob", name: "软萌团子 2.0", gender: "female", languages: ["zh"], scenes: ["customer-service"], tone: "rose", description: "软萌、客服。", previewUrl: mp3ById("ICL_uranus_zh_female_ruanmengtuanzi_tob"), avatarUrl: resolveAvatarUrl("ICL_uranus_zh_female_ruanmengtuanzi_tob", "软萌团子 2.0") },
  { id: "ICL_uranus_zh_female_nuanxinqianqian_tob", name: "暖心茜茜 2.0", gender: "female", languages: ["zh"], scenes: ["customer-service"], tone: "amber", description: "暖心、客服。", previewUrl: mp3ById("ICL_uranus_zh_female_nuanxinqianqian_tob"), avatarUrl: resolveAvatarUrl("ICL_uranus_zh_female_nuanxinqianqian_tob", "暖心茜茜 2.0") },
  { id: "ICL_uranus_zh_female_kaixinxiaohong_tob", name: "开心小鸿 2.0", gender: "female", languages: ["zh"], scenes: ["customer-service"], tone: "amber", description: "简洁亲切、客服。", previewUrl: mp3ById("ICL_uranus_zh_female_kaixinxiaohong_tob"), avatarUrl: resolveAvatarUrl("ICL_uranus_zh_female_kaixinxiaohong_tob", "开心小鸿 2.0") },
  { id: "ICL_uranus_zh_female_chunchenvsheng_tob", name: "纯澈女生 2.0", gender: "female", languages: ["zh"], scenes: ["general"], tone: "mint", description: "清澈、叙事。", previewUrl: mp3ById("ICL_uranus_zh_female_chunchenvsheng_tob"), avatarUrl: resolveAvatarUrl("ICL_uranus_zh_female_chunchenvsheng_tob", "纯澈女生 2.0") },
  { id: "ICL_uranus_zh_female_wenrounvshen_tob", name: "温柔女神 2.0", gender: "female", languages: ["zh"], scenes: ["general"], tone: "rose", description: "温柔、通用。", previewUrl: mp3ById("ICL_uranus_zh_female_wenrounvshen_tob"), avatarUrl: resolveAvatarUrl("ICL_uranus_zh_female_wenrounvshen_tob", "温柔女神 2.0") },
  { id: "ICL_uranus_zh_female_zhixinjiejie_tob", name: "知心姐姐 2.0", gender: "female", languages: ["zh"], scenes: ["general"], tone: "blue", description: "通用、友好。", previewUrl: mp3ById("ICL_uranus_zh_female_zhixinjiejie_tob"), avatarUrl: resolveAvatarUrl("ICL_uranus_zh_female_zhixinjiejie_tob", "知心姐姐 2.0") },
  { id: "ICL_uranus_zh_female_yuanqitianmei_tob", name: "元气甜妹 2.0", gender: "female", languages: ["zh"], scenes: ["general"], tone: "amber", description: "元气、通用。", previewUrl: mp3ById("ICL_uranus_zh_female_yuanqitianmei_tob"), avatarUrl: resolveAvatarUrl("ICL_uranus_zh_female_yuanqitianmei_tob", "元气甜妹 2.0") },
  { id: "ICL_uranus_zh_female_tiexinguimi_tob", name: "贴心闺蜜 2.0", gender: "female", languages: ["zh"], scenes: ["general"], tone: "rose", description: "友好、通用。", previewUrl: mp3ById("ICL_uranus_zh_female_tiexinguimi_tob"), avatarUrl: resolveAvatarUrl("ICL_uranus_zh_female_tiexinguimi_tob", "贴心闺蜜 2.0") },
  { id: "ICL_uranus_zh_female_tiexinmeimei_tob", name: "贴心妹妹 2.0", gender: "female", languages: ["zh"], scenes: ["general"], tone: "rose", description: "友好、通用。", previewUrl: mp3ById("ICL_uranus_zh_female_tiexinmeimei_tob"), avatarUrl: resolveAvatarUrl("ICL_uranus_zh_female_tiexinmeimei_tob", "贴心妹妹 2.0") },
  { id: "ICL_uranus_zh_female_wenroubaiyueguang_tob", name: "温柔白月光 2.0", gender: "female", languages: ["zh"], scenes: ["general"], tone: "rose", description: "友好、通用。", previewUrl: mp3ById("ICL_uranus_zh_female_wenroubaiyueguang_tob"), avatarUrl: resolveAvatarUrl("ICL_uranus_zh_female_wenroubaiyueguang_tob", "温柔白月光 2.0") },
  { id: "ICL_uranus_zh_female_chuliannvyou_tob", name: "初恋女友 2.0", gender: "female", languages: ["zh"], scenes: ["general"], tone: "rose", description: "友好、通用。", previewUrl: mp3ById("ICL_uranus_zh_female_chuliannvyou_tob"), avatarUrl: resolveAvatarUrl("ICL_uranus_zh_female_chuliannvyou_tob", "初恋女友 2.0") },
  { id: "ICL_uranus_zh_female_zhixingwenwan_tob", name: "知性温婉 2.0", gender: "female", languages: ["zh"], scenes: ["general"], tone: "blue", description: "专业、通用。", previewUrl: mp3ById("ICL_uranus_zh_female_zhixingwenwan_tob"), avatarUrl: resolveAvatarUrl("ICL_uranus_zh_female_zhixingwenwan_tob", "知性温婉 2.0") },
  { id: "ICL_uranus_zh_male_yangyang_tob", name: "炀炀 2.0", gender: "male", languages: ["zh"], scenes: ["general"], tone: "blue", description: "友好、通用。", previewUrl: mp3ById("ICL_uranus_zh_male_yangyang_tob"), avatarUrl: resolveAvatarUrl("ICL_uranus_zh_male_yangyang_tob", "炀炀 2.0") },
  { id: "ICL_uranus_zh_male_chunhoudiyin_tob", name: "醇厚低音 2.0", gender: "male", languages: ["zh"], scenes: ["narration", "general"], tone: "slate", description: "低音、叙事。", previewUrl: mp3ById("ICL_uranus_zh_male_chunhoudiyin_tob"), avatarUrl: resolveAvatarUrl("ICL_uranus_zh_male_chunhoudiyin_tob", "醇厚低音 2.0") },
  { id: "ICL_uranus_zh_male_paoxiaoxiaoge_tob", name: "咆哮小哥 2.0", gender: "male", languages: ["zh"], scenes: ["narration"], tone: "amber", description: "有嘶、叙事。", previewUrl: mp3ById("ICL_uranus_zh_male_paoxiaoxiaoge_tob"), avatarUrl: resolveAvatarUrl("ICL_uranus_zh_male_paoxiaoxiaoge_tob", "咆哮小哥 2.0") },
  { id: "ICL_uranus_zh_male_diyinchenyu_tob", name: "低音沉郁 2.0", gender: "male", languages: ["zh"], scenes: ["narration"], tone: "slate", description: "低音、叙事。", previewUrl: mp3ById("ICL_uranus_zh_male_diyinchenyu_tob"), avatarUrl: resolveAvatarUrl("ICL_uranus_zh_male_diyinchenyu_tob", "低音沉郁 2.0") },
  { id: "ICL_uranus_zh_female_qingxinshaonv_tob", name: "倾心少女 2.0", gender: "female", languages: ["zh"], scenes: ["narration", "general"], tone: "rose", description: "视频配音。", previewUrl: mp3ById("ICL_uranus_zh_female_qingxinshaonv_tob"), avatarUrl: resolveAvatarUrl("ICL_uranus_zh_female_qingxinshaonv_tob", "倾心少女 2.0") },
  { id: "ICL_uranus_zh_female_tianmeitaozi_tob", name: "甜美桃子 ICL", gender: "female", languages: ["zh"], scenes: ["general", "character"], tone: "amber", description: "ICL 优化版代口。", previewUrl: mp3ById("ICL_uranus_zh_female_tianmeitaozi_tob"), avatarUrl: resolveAvatarUrl("ICL_uranus_zh_female_tianmeitaozi_tob", "甜美桃子 ICL") },
];

// 火山公开音色目录同时包含 2.0、1.0 和客服/角色音色。样音文件不是稳定接口，
// 因此扩展目录只保存 speaker ID，样音失败时统一交给 Rust TTS 实时合成。
const EXPANDED_VOICE_SEEDS = [
  ["zh_female_cancan_mars_bigtts", "灿灿"], ["zh_female_qingxinnvsheng_mars_bigtts", "清新女声"],
  ["zh_female_shuangkuaisisi_moon_bigtts", "爽快思思"], ["zh_male_wennuanahu_moon_bigtts", "温暖阿虎"],
  ["zh_male_shaonianzixin_moon_bigtts", "少年梓辛"], ["zh_female_zhixingnvsheng_mars_bigtts", "知性女声"],
  ["zh_male_qingshuangnanda_mars_bigtts", "清爽男大"], ["zh_female_linjianvhai_moon_bigtts", "邻家女孩"],
  ["zh_male_yuanboxiaoshu_moon_bigtts", "渊博小叔"], ["zh_male_yangguangqingnian_moon_bigtts", "阳光青年"],
  ["zh_female_tianmeixiaoyuan_moon_bigtts", "甜美小源"], ["zh_female_qingchezizi_moon_bigtts", "清澈梓梓"],
  ["zh_male_jieshuoxiaoming_moon_bigtts", "解说小明"], ["zh_female_kailangjiejie_moon_bigtts", "开朗姐姐"],
  ["zh_male_linjiananhai_moon_bigtts", "邻家男孩"], ["zh_female_tianmeiyueyue_moon_bigtts", "甜美悦悦"],
  ["zh_female_xinlingjitang_moon_bigtts", "心灵鸡汤"], ["en_male_smith_mars_bigtts", "Smith"],
  ["en_female_anna_mars_bigtts", "Anna"], ["en_male_adam_mars_bigtts", "Adam"],
  ["en_female_sarah_mars_bigtts", "Sarah"], ["en_male_dryw_mars_bigtts", "Dryw"],
  ["multi_male_jingqiangkanye_moon_bigtts", "京腔侃爷"], ["multi_female_shuangkuaisisi_moon_bigtts", "多语思思"],
  ["multi_male_wanqudashu_moon_bigtts", "顽酷大叔"], ["multi_female_gaolengyujie_moon_bigtts", "多语御姐"],
  ["zh_male_jingqiangkanye_moon_bigtts", "京腔侃爷"], ["zh_female_wanwanxiaohe_moon_bigtts", "弯弯小何"],
  ["zh_female_wanqudashu_moon_bigtts", "顽酷大叔"], ["zh_female_daimengchuanmei_moon_bigtts", "呆萌川妹"],
  ["zh_male_guozhoudege_moon_bigtts", "果州大哥"], ["zh_male_beijingxiaoye_moon_bigtts", "北京小爷"],
  ["zh_male_haoyuxiaoge_moon_bigtts", "皓宇小哥"], ["zh_male_guangxiyuanzhou_moon_bigtts", "广西远舟"],
  ["zh_female_meituojieer_moon_bigtts", "美妥姐儿"], ["zh_male_yuzhouzixuan_moon_bigtts", "宇宙梓轩"],
  ["zh_male_naiqimengwa_mars_bigtts", "奶气萌娃"], ["zh_female_popo_mars_bigtts", "婆婆"],
  ["zh_female_gaolengyujie_moon_bigtts", "高冷御姐"], ["zh_male_aojiaobazong_moon_bigtts", "傲娇霸总"],
  ["zh_female_meilinvyou_moon_bigtts", "魅力女友"], ["zh_male_shenyeboke_moon_bigtts", "深夜播客"],
  ["zh_female_sajiaonvyou_moon_bigtts", "撒娇女友"], ["zh_female_yuanqinvyou_moon_bigtts", "元气女友"],
  ["ICL_zh_female_bingruoshaonv_tob", "冰若少女"], ["ICL_zh_female_huoponvhai_tob", "活泼女孩"],
  ["zh_male_dongfanghaoran_moon_bigtts", "东方浩然"], ["ICL_zh_female_heainainai_tob", "和蔼奶奶"],
  ["ICL_zh_female_linjuayi_tob", "邻家依依"], ["zh_female_wenrouxiaoya_moon_bigtts", "温柔小雅"],
  ["zh_male_tiancaitongsheng_mars_bigtts", "天才童声"], ["zh_male_sunwukong_mars_bigtts", "孙悟空"],
  ["zh_male_xionger_mars_bigtts", "熊二"], ["zh_female_peiqi_mars_bigtts", "佩奇猪"],
  ["zh_female_wuzetian_mars_bigtts", "武则天"], ["zh_female_gujie_mars_bigtts", "古街"],
  ["zh_female_yingtaowanzi_mars_bigtts", "樱桃丸子"], ["zh_male_chunhui_mars_bigtts", "春晖"],
  ["zh_female_shaoergushi_mars_bigtts", "少儿故事"], ["zh_male_silang_mars_bigtts", "四郎"],
  ["zh_male_jieshuonansheng_mars_bigtts", "解说男声"], ["zh_female_jitangmeimei_mars_bigtts", "鸡汤妹妹"],
  ["zh_female_tiexinnvsheng_mars_bigtts", "贴心女声"], ["zh_female_qiaopinvsheng_mars_bigtts", "俏皮女声"],
  ["zh_female_mengyatou_mars_bigtts", "萌丫头"], ["zh_male_changtianyi_mars_bigtts", "常天一"],
  ["zh_male_ruyaqingnian_mars_bigtts", "儒雅青年"], ["zh_male_baqiqingshu_mars_bigtts", "霸气青叔"],
  ["zh_male_qingcang_mars_bigtts", "擎苍"], ["zh_female_gufengshaoyu_mars_bigtts", "古风少御"],
  ["zh_female_wenroushunv_mars_bigtts", "温柔淑女"], ["zh_male_sunwukong_uranus_bigtts", "猴哥 2.0"],
  ["zh_female_yingyujiaoxue_uranus_bigtts", "Tina老师 2.0"], ["zh_female_kefunvsheng_uranus_bigtts", "暖阳女声 2.0"],
  ["zh_female_xiaoxue_uranus_bigtts", "儿童绘本 2.0"], ["zh_male_dayi_uranus_bigtts", "大壹 2.0"],
  ["zh_female_mizai_uranus_bigtts", "咪仔 2.0"], ["zh_female_jitangnv_uranus_bigtts", "鸡汤女 2.0"],
  ["zh_female_liuchangnv_uranus_bigtts", "流畅女声 2.0"], ["zh_male_ruyayichen_uranus_bigtts", "儒雅逸辰 2.0"],
  ["saturn_zh_female_keainvsheng_tob", "可爱女生"], ["saturn_zh_female_qingyingduoduo_cs_tob", "轻盈朵朵"],
  ["en_male_tim_uranus_bigtts", "Tim"],
] as const;

function createExpandedVoice([id, name]: (typeof EXPANDED_VOICE_SEEDS)[number]): TtsVoiceCatalogItem {
  const isFemale = id.includes("_female_");
  const isLegacy = id.endsWith("_mars_bigtts") || id.endsWith("_moon_bigtts") || id.startsWith("ICL_");
  const languages: TtsVoiceLanguage[] = id.startsWith("en_")
    ? ["en"]
    : id.startsWith("multi_")
      ? ["zh", "ja", "es"]
      : ["zh"];

  return {
    id,
    name,
    gender: isFemale ? "female" : "male",
    languages,
    scenes: id.startsWith("ICL_") ? ["customer-service", "general"] : isLegacy ? ["narration", "general"] : ["general"],
    tone: isFemale ? "rose" : "slate",
    description: isLegacy ? "火山公开音色，适合视频配音、解说和口播。" : "豆包语音 2.0 音色，支持情感和语气控制。",
    previewUrl: mp3ById(id),
    avatarUrl: resolveAvatarUrl(id, name),
  };
}

const catalogIds = new Set(TTS_VOICE_CATALOG.map((voice) => voice.id));
for (const voice of EXPANDED_VOICE_SEEDS.map(createExpandedVoice)) {
  if (!catalogIds.has(voice.id)) {
    TTS_VOICE_CATALOG.push(voice);
    catalogIds.add(voice.id);
  }
}



export function findTtsVoice(speakerId: string) {
  return TTS_VOICE_CATALOG.find((voice) => voice.id === speakerId) ?? null;
}
