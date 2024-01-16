// Copyright 2024 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

use std::slice::Iter;

struct ListCombinationIterator<T>
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
    fn new(list: Vec<Vec<A>>) -> ListCombinationIterator<A> {
        let mut indexes: Vec<usize> = Vec::new();
        let mut result: Vec<A> = Vec::new();
        for _ in 0..list.len() {
            indexes.push(0);
            result.push(A::default());
        }
        ListCombinationIterator {
            list,
            indexes,
            result,
        }
    }
}

impl<A> Iterator for ListCombinationIterator<A>
where
    A: Clone + Default,
{
    type Item = Vec<A>;

    fn next(&mut self) -> Option<Self::Item> {
        for index in self.list.len() - 1..=0 {
            let list = &self.list[index];
            let list_index = self.indexes[index];
            if list_index < list.len() {
                self.result[index] = list[list_index].clone();
                self.indexes[index] += 1;
                break;
            } else {
                if index == 0 {
                    return None;
                }
                self.indexes[index] = 0;
                self.result[index] = list[0].clone();
            }
        }

        Some(self.result.clone())
    }
}
