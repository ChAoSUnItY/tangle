#[derive(Debug, Clone)]
pub struct Alias<'src> {
    pub alias: &'src str,
    pub source_span: &'src str,
    pub disabled: bool
}

impl<'src> Alias<'src> {
    pub fn new(alias: &'src str, source_span: &'src str) -> Self {
        Self {
            alias,
            source_span,
            disabled: false,
        }
    }
}

#[derive(Debug)]
pub struct Macro<'src> {
    pub name: &'src str,
    pub parameters: Vec<Alias<'src>>,
    pub source_span: &'src str,
}

impl<'src> Macro<'src> {
    pub fn new(name: &'src str, parameters: Vec<Alias<'src>>, source_span: &'src str) -> Self {
        Self {
            name,
            parameters,
            source_span
        }
    }
}
