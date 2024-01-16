// Copyright 2024 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[derive(Debug, PartialEq)]
pub(crate) struct ListCombinationIterator<T>
where
    T: Clone + Default,
{
    list: Vec<Vec<T>>,
    indexes: Vec<usize>,
    result: Vec<T>,
}

impl<A> ListCombinationIterator<A>
where
    A: Clone + Default,
{
    pub(crate) fn new(list: Vec<Vec<A>>) -> Option<ListCombinationIterator<A>> {
        if list.is_empty() {
            return None;
        }
        for sublist in list.iter() {
            if sublist.is_empty() {
                return None;
            }
        }
        let mut indexes: Vec<usize> = Vec::new();
        let mut result: Vec<A> = Vec::new();
        for sublist in list.iter() {
            indexes.push(0);
            result.push(sublist[0].clone());
        }
        Some(ListCombinationIterator {
            list,
            indexes,
            result,
        })
    }
}

impl<A> Iterator for ListCombinationIterator<A>
where
    A: Clone + Default,
{
    type Item = Vec<A>;

    fn next(&mut self) -> Option<Self::Item> {
        for index in (0..self.list.len()).rev() {
            let sublist = &self.list[index];
            let list_index = self.indexes[index];
            if list_index < sublist.len() {
                self.result[index] = sublist[list_index].clone();
                let indexes_length = self.indexes.len();
                self.indexes[indexes_length - 1] += 1;
                break;
            } else {
                if index == 0 {
                    return None;
                }
                self.indexes[index] = 0;
                self.result[index] = sublist[0].clone();
                self.indexes[index - 1] += 1;
            }
        }

        Some(self.result.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_empty_list() {
        let list = vec![vec![1, 2], Vec::<i32>::new(), vec![5, 6]];
        assert_eq!(ListCombinationIterator::<i32>::new(list), None);
    }

    #[test]
    fn has_no_lists() {
        let list = Vec::<Vec<i32>>::new();
        assert_eq!(ListCombinationIterator::new(list), None);
    }

    #[test]
    fn test_list_combination_iterator() {
        let list = vec![vec![1, 2], vec![3, 4], vec![5, 6]];
        let mut iter = ListCombinationIterator::new(list).unwrap();
        assert_eq!(iter.next(), Some(vec![1, 3, 5]));
        assert_eq!(iter.next(), Some(vec![1, 3, 6]));
        assert_eq!(iter.next(), Some(vec![1, 4, 5]));
        assert_eq!(iter.next(), Some(vec![1, 4, 6]));
        assert_eq!(iter.next(), Some(vec![2, 3, 5]));
        assert_eq!(iter.next(), Some(vec![2, 3, 6]));
        assert_eq!(iter.next(), Some(vec![2, 4, 5]));
        assert_eq!(iter.next(), Some(vec![2, 4, 6]));
        assert_eq!(iter.next(), None);
    }
}
