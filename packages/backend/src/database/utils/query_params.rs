use postgres_types::ToSql;

/// Utility to keep track of query parameters for database queries
pub struct QueryParams {
    params: Vec<Box<dyn ToSql + Sync + Send>>,
}

impl QueryParams {
    /// Creates a new instance of `QueryParams`.
    pub fn new() -> Self {
        Self { params: vec![] }
    }

    /// Adds a new parameter to the query, and return its index
    pub fn push<T: ToSql + Sync + Send + 'static>(&mut self, value: T) -> usize {
        self.params.push(Box::new(value));
        self.params.len()
    }

    /// Returns a list of references to the query parameters.
    pub fn as_refs(&self) -> Vec<&(dyn ToSql + Sync)> {
        self.params
            .iter()
            .map(|p| &**p as &(dyn ToSql + Sync))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keeps_track_of_indexes() {
        let mut params = QueryParams::new();
        let index1 = params.push("test1");
        let index2 = params.push("test2");
        assert_eq!(index1, 1);
        assert_eq!(index2, 2);
    }

    #[test]
    fn test_as_refs() {
        let mut params = QueryParams::new();
        params.push("test1");
        params.push("test2");

        let refs = params.as_refs();
        assert_eq!(refs.len(), 2);
    }
}
