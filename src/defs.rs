use crate::source::SourceSegments;

#[derive(Debug, Clone)]
pub struct Alias<'src> {
    pub alias: SourceSegments<'src>,
    pub replacement: SourceSegments<'src>,
    pub disabled: bool,
}

impl<'src> Alias<'src> {
    pub fn new(alias: SourceSegments<'src>, replacement: SourceSegments<'src>) -> Self {
        Self {
            alias,
            replacement,
            disabled: false,
        }
    }
}

#[derive(Debug)]
pub struct Macro<'src> {
    pub name: SourceSegments<'src>,
    pub parameters: Vec<Alias<'src>>,
    pub is_variadic: bool,
    pub source_span: SourceSegments<'src>,
}

impl<'src> Macro<'src> {
    pub fn new(
        name: SourceSegments<'src>,
        parameters: Vec<Alias<'src>>,
        is_variadic: bool,
        source_span: SourceSegments<'src>,
    ) -> Self {
        Self {
            name,
            parameters,
            is_variadic,
            source_span,
        }
    }
}
