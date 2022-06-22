pub enum RuntimeType {
    Dev,
    Gafi,
    Gaki,
    Gari,
}

impl Default for RuntimeType {
    fn default() -> RuntimeType {
        RuntimeType::Dev
    }
}
