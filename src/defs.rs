#[derive(Debug, Clone)]
pub struct Alias {
    pub alias: String,
    pub replacement: String,
    pub disabled: bool,
}

impl Alias {
    pub fn new(alias: String, replacement: String) -> Self {
        Self {
            alias,
            replacement,
            disabled: false,
        }
    }
}

#[derive(Debug)]
pub struct Macro {
    pub name: String,
    pub parameters: Vec<Alias>,
    pub is_variadic: bool,
    pub source_span: String,
}

impl Macro {
    pub fn new(
        name: String,
        parameters: Vec<Alias>,
        is_variadic: bool,
        source_span: String,
    ) -> Self {
        Self {
            name,
            parameters,
            is_variadic,
            source_span,
        }
    }
}
