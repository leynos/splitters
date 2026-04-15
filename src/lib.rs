//! Library entry points for the `splitters` package.

/// Return the current stub greeting used by the binary entry point.
#[must_use]
pub const fn greeting() -> &'static str { "Hello from Splitters!" }

#[cfg(test)]
mod tests {
    //! Tests for the stub library behaviour.

    use super::greeting;

    #[test]
    fn greeting_matches_stub_output() {
        assert_eq!(greeting(), "Hello from Splitters!");
    }
}
