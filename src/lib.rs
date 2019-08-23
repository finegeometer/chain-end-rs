/// One end of a chain of values.
#[must_use = "If a chain end is dropped, the chain is leaked."]
pub struct ChainEnd<T>(*mut Node<T>);

struct Node<T> {
    ptr: usize,
    data: Data<T>,
}

impl<T> Node<T> {
    fn end_pair() -> (*mut Self, *mut Self) {
        let e1 = Box::into_raw(Box::new(Self {
            ptr: 0,
            data: Data { uninit: () },
        }));
        let e2 = Box::into_raw(Box::new(Self {
            ptr: 0,
            data: Data { uninit: () },
        }));

        unsafe {
            (*e1).data = Data { other_end: e2 };
            (*e2).data = Data { other_end: e1 };
        }

        (e1, e2)
    }
}

union Data<T> {
    other_end: *mut Node<T>, // End node
    value: *mut T,           // Interior Node
    uninit: (),
}

impl<T> ChainEnd<T> {
    /// Create a new chain, returning the two ends.
    pub fn new(iter: impl IntoIterator<Item = T>) -> (Self, Self) {
        unsafe {
            let (e1, e2) = Node::end_pair();

            let mut last_node: *mut Node<T> = e1;

            for value in iter {
                let value: *mut T = Box::into_raw(Box::new(value));

                let this_node: *mut Node<T> = Box::into_raw(Box::new(Node {
                    ptr: last_node as usize,
                    data: Data { value },
                }));
                (*last_node).ptr ^= this_node as usize;

                last_node = this_node;
            }

            (*e2).ptr = last_node as usize;
            (*last_node).ptr ^= e2 as usize;

            (ChainEnd(e1), ChainEnd(e2))
        }
    }

    /// Join two chain ends.
    /// If they were two ends of the same chain, return the contents of the loop this creates.
    /// If they were ends of different chains, return None.
    pub fn connect(self, other: Self) -> Option<impl Iterator<Item = T>> {
        unsafe {
            if (*self.0).data.other_end == other.0 {
                let this_end: *mut Node<T> = other.0;

                let other_end: *mut Node<T> = (*this_end).data.other_end;
                let first_node: *mut Node<T> = (*this_end).ptr as *mut Node<T>;

                (*first_node).ptr ^= this_end as usize;

                Box::from_raw(this_end);

                Some(ChainIterator {
                    end: other_end,
                    current_node: first_node,
                })
            } else {
                let a = self.0;
                let b = other.0;

                let a_end: *mut Node<T> = (*a).data.other_end;
                let b_end: *mut Node<T> = (*b).data.other_end;

                (*a_end).data.other_end = b_end;
                (*b_end).data.other_end = a_end;

                let a_node: *mut Node<T> = (*a).ptr as *mut Node<T>;
                let b_node: *mut Node<T> = (*b).ptr as *mut Node<T>;

                (*a_node).ptr ^= a as usize ^ b_node as usize;
                (*b_node).ptr ^= b as usize ^ a_node as usize;

                Box::from_raw(a);
                Box::from_raw(b);

                None
            }
        }
    }
}

struct ChainIterator<T> {
    end: *mut Node<T>,
    current_node: *mut Node<T>,
}

impl<T> Drop for ChainIterator<T> {
    fn drop(&mut self) {
        unsafe {
            while let Some(_) = self.next() {}
            Box::from_raw(self.end);
        };
    }
}

impl<T> Iterator for ChainIterator<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        if self.current_node == self.end {
            return None;
        }
        unsafe {
            let this_node: *mut Node<T> = self.current_node;
            let next_node: *mut Node<T> = (*this_node).ptr as *mut Node<T>;

            let value: T = *Box::from_raw((*this_node).data.value);

            (*next_node).ptr ^= this_node as usize;

            Box::from_raw(this_node);

            self.current_node = next_node;

            Some(value)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ChainEnd;

    #[test]
    fn it_works() {
        let (a, b) = ChainEnd::new(0..3);
        let (c, d) = ChainEnd::new(3..6);

        // We now have these chains:
        // a - 0 - 1 - 2 - b
        // c - 3 - 4 - 5 - d

        // We connect b to d, creating one long chain.
        assert!(b.connect(d).is_none());

        // We now have this chain:
        // a - 0 - 1 - 2 - 5 - 4 - 3 - c

        // We connect c to a, creating a loop. The contents of the loop are returned.
        assert_eq!(
            c.connect(a).unwrap().collect::<Vec<_>>(),
            vec![0, 1, 2, 5, 4, 3]
        );
    }
}
