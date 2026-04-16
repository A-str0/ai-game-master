pub(super) fn load_llm_config() -> (String, String) {
    let api_key = std::env::var("LLM_API_KEY").unwrap_or_else(|_| String::from("API_KEY"));
    let model = std::env::var("LLM_MODEL")
        .unwrap_or_else(|_| String::from("nvidia/nemotron-3-super-120b-a12b:free"));

    (api_key, model)
}
