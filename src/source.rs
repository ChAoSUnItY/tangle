use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSegments<'src> {
    segments: Vec<&'src [u8]>,
    len: usize,
}

impl<'src> SourceSegments<'src> {
    #[inline]
    pub fn new(segments: &[&'src [u8]]) -> Self {
        Self {
            segments: segments.to_vec(),
            len: segments.into_iter().map(|segment| segment.len()).sum(),
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn push_segment(&mut self, segments: &SourceSegments<'src>) {
        self.segments.extend_from_slice(&segments.segments);
        self.len += segments.len();
    }

    #[inline]
    pub fn push_span(&mut self, span: &'src str) {
        self.segments.push(span.as_bytes());
        self.len += span.len();
    }

    pub fn index(&self, index: usize) -> Option<u8> {
        let mut len_acc = 0;

        for segment in self.segments.iter() {
            if index >= len_acc && index < len_acc + segment.len() {
                return Some(segment[index - len_acc]);
            }

            len_acc += segment.len();
        }

        None
    }

    pub fn index_range(&self, range: Range<usize>) -> SourceSegments<'src> {
        if range.start > range.end || range.end > self.len() {
            return SourceSegments::new(&[]);
        }

        let mut len_acc = 0;
        let mut segment_builder = vec![];

        for segment in self.segments.iter() {
            let (lb, rb) = (len_acc, len_acc + segment.len());

            if (lb..rb).contains(&range.start) {
                if range.end > rb {
                    segment_builder.push(&segment[(range.start - lb)..]);
                } else {
                    segment_builder.push(&segment[(range.start - lb)..(range.end - lb)]);
                }
            } else if range.start < lb {
                if range.end > rb {
                    segment_builder.push(&segment[..]);
                } else if range.end > lb {
                    segment_builder.push(&segment[..(range.end - lb)]);
                }
            }

            len_acc += segment.len();
        }

        SourceSegments::new(&segment_builder)
    }
}

impl<'src, 'other> PartialEq<&'other str> for SourceSegments<'src> {
    fn eq(&self, other: &&'other str) -> bool {
        other.as_bytes()
            == self
                .segments
                .iter()
                .flat_map(|&segment| segment)
                .copied()
                .collect::<Vec<_>>()
    }
}

impl<'src, 'other> PartialEq<SourceSegments<'src>> for &'other str {
    fn eq(&self, other: &SourceSegments<'src>) -> bool {
        self.as_bytes()
            == other
                .segments
                .iter()
                .flat_map(|&segment| segment)
                .copied()
                .collect::<Vec<_>>()
    }
}

#[cfg(test)]
mod test {
    use std::ops::Range;

    use super::SourceSegments;
    use rand::Rng;

    fn gen_range(string: &str) -> Range<usize> {
        let mut rng = rand::thread_rng();
        let lb = rng.gen_range(0..string.len());
        let ub = rng.gen_range(lb..string.len());

        lb..ub
    }

    #[test]
    fn test_source_segment_len() {
        let ss = SourceSegments::new(&[b"ask", b"me", b"everything"]);

        assert_eq!(ss.len(), 15);
    }

    #[test]
    fn test_source_segment_index() {
        let ss = SourceSegments::new(&[b"ask", b"me", b"everything"]);

        assert_eq!(ss.index(3), Some(b'm'));
        assert_eq!(ss.index(15), None);
    }

    #[test]
    fn test_source_segment_index_range() {
        let ss = SourceSegments::new(&[b"ask", b"me", b"everything"]);

        assert_eq!(ss.index_range(1..4), SourceSegments::new(&[b"sk", b"m"]));
        assert_eq!(ss.index_range(14..15), SourceSegments::new(&[b"g"]));
    }

    #[test]
    fn test_source_segment_index_range_random_slice() {
        let os = "askmeeverything";
        let ss = SourceSegments::new(&[b"ask", b"me", b"everything"]);

        for _ in 0..1000 {
            let range = gen_range(os);

            assert_eq!(&os[range.clone()], ss.index_range(range));
        }
    }
}
