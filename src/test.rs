//! Helper class for testing.

use crate::{CodeLocation, CodeLocationStack, Result};
use std::collections::HashMap;
use std::fmt;

// Test is up here to avoid the line number moving around.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tag_location() {
        let mut fix = Fixture::default();
        fix.tag_location("tag", CodeLocation::here());
        assert_eq!(
            *fix.get_location("tag"),
            CodeLocation::new("src/test.rs", 15)
        );
    }
}

#[allow(dead_code)]
#[derive(Default)]
pub struct Fixture {
    code_locations: HashMap<&'static str, CodeLocation>,
}

#[allow(dead_code)]
impl Fixture {
    pub fn tag_location(&mut self, tag: &'static str, loc: CodeLocation) {
        self.code_locations.insert(tag, loc);
    }

    pub fn get_location(&self, tag: &'static str) -> &CodeLocation {
        self.code_locations.get(tag).unwrap()
    }

    pub fn assert_stack_matches_tags(&self, stack: &CodeLocationStack, tags: &[&'static str]) {
        let tags_to_locations =
            CodeLocationStack(tags.iter().map(|t| *self.get_location(t)).collect());
        assert_eq!(stack, &tags_to_locations);
    }

    pub fn assert_result_has_stack<T: fmt::Debug, E: fmt::Debug>(
        &self,
        result: Result<T, E>,
        tags: &[&'static str],
    ) {
        let (_err, stack) = result.err_trace().unwrap();
        self.assert_stack_matches_tags(&stack, tags);
    }
}
