use crate::provider::model::ProviderCategory;

pub struct Preset {
    pub category: ProviderCategory,
    pub label: &'static str,
    pub base_url: &'static str,
    pub models: &'static [&'static str],
}

/// Built-in OpenAI-compatible presets. The user can still edit base_url / models per provider.
pub fn presets() -> Vec<Preset> {
    vec![
        Preset {
            category: ProviderCategory::Openai,
            label: "OpenAI",
            base_url: "https://api.openai.com/v1",
            models: &["gpt-4o-mini", "gpt-4o", "gpt-4.1-mini"],
        },
        Preset {
            category: ProviderCategory::Deepseek,
            label: "DeepSeek",
            base_url: "https://api.deepseek.com/v1",
            models: &["deepseek-chat", "deepseek-reasoner"],
        },
        Preset {
            category: ProviderCategory::Siliconflow,
            label: "硅基流动 (SiliconFlow)",
            base_url: "https://api.siliconflow.cn/v1",
            models: &["Qwen/Qwen2.5-7B-Instruct", "deepseek-ai/DeepSeek-V3"],
        },
        Preset {
            category: ProviderCategory::Qwen,
            label: "阿里云百炼 (Qwen)",
            base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1",
            models: &["qwen-plus", "qwen-turbo", "qwen-max"],
        },
        Preset {
            category: ProviderCategory::Moonshot,
            label: "Moonshot (Kimi)",
            base_url: "https://api.moonshot.cn/v1",
            models: &["moonshot-v1-8k", "moonshot-v1-32k"],
        },
        Preset {
            category: ProviderCategory::Zhipu,
            label: "智谱 (GLM)",
            base_url: "https://open.bigmodel.cn/api/paas/v4",
            models: &["glm-4-flash", "glm-4", "glm-4-air"],
        },
        Preset {
            category: ProviderCategory::Ollama,
            label: "Ollama / LM Studio (本地)",
            base_url: "http://localhost:11434/v1",
            models: &[],
        },
        Preset {
            category: ProviderCategory::Custom,
            label: "自定义",
            base_url: "",
            models: &[],
        },
    ]
}
