use std::fmt::{Debug, Display};

use crate::db::model::FieldSort;

pub trait RemoveReplaceExisting<V: PartialEq, T: FieldSort<V>> {
    fn remove_existing(&self, from: &[T]) -> Vec<T>;
    fn replace_existing(&self, from: &[T]) -> Vec<T>;
}

impl<V: PartialEq + Display, T: FieldSort<V> + Clone + Debug> RemoveReplaceExisting<V, T> for Vec<T> {
    fn remove_existing(&self, from: &[T]) -> Vec<T> {
        self.iter()
            .filter(|&elt| {
                !from
                    .iter()
                    .any(|v| v.sort_by_value() == elt.sort_by_value())
            })
            .cloned()
            .collect()
    }

    fn replace_existing(&self, replace_with: &[T]) -> Vec<T> {
        self.iter()
            .map(|elt| {
                replace_with
                    .iter()
                    .find(|&v| v.sort_by_value() == elt.sort_by_value())
                .unwrap_or(elt)
                .clone()
            })
            .collect()
    }
    
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(PartialEq, Clone, Debug)]
    struct DummyReplace {
        field: String,
        number: i32,
    }

    impl FieldSort<String> for DummyReplace {
        fn sort_by_value(&self) -> String {
            self.field.clone()
        }
    }

    #[test]
    fn test_can_replace_existing() {
        let vec1 = vec![
            DummyReplace {
                field: "a".to_string(),
                number: 0,
            },
            DummyReplace {
                field: "b".to_string(),
                number: 1,
            },
        ];
        let vec2 = vec![
            DummyReplace {
                field: "a".to_string(),
                number: 420,
            },
            DummyReplace {
                field: "c".to_string(),
                number: 2,
            },
        ];
        let res = vec1.replace_existing(&vec2);
        assert_eq!(
            res,
            vec![DummyReplace {
                field: "a".to_string(),
                number: 420,
            }, DummyReplace {
                field: "b".to_string(),
                number: 1,
            }]
        );
    }

    #[derive(PartialEq, Clone, Debug)]
    struct Dummy {
        field: String,
    }

    impl FieldSort<String> for Dummy {
        fn sort_by_value(&self) -> String {
            self.field.clone()
        }
    }

    #[test]
    fn test_can_remove_existing1() {
        let vec1 = vec![
            Dummy {
                field: "a".to_string(),
            },
            Dummy {
                field: "b".to_string(),
            },
        ];
        let vec2 = vec![
            Dummy {
                field: "a".to_string(),
            },
            Dummy {
                field: "c".to_string(),
            },
        ];
        let res = vec1.remove_existing(&vec2);
        assert_eq!(
            res,
            vec![Dummy {
                field: "b".to_string(),
            }]
        );
    }

    #[test]
    fn test_can_remove_existing() {
        let vec1 = vec![Dummy {
            field: "a".to_string(),
        }];
        let vec2 = vec![];
        let res = vec1.remove_existing(&vec2);
        assert_eq!(
            res,
            vec![Dummy {
                field: "a".to_string()
            }]
        );
    }
}
