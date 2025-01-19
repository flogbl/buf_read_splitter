use std::{
    alloc::{alloc_zeroed, dealloc, realloc, Layout},
    cmp,
    fmt::Debug,
};

/// To manage a generic Result
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// FIFO implementation, circular buffer underlying, capacity can be modifiable.
/// It is not for public access because unsure to use as-is since there's few checks (the effort has to come from the calling algorithm)
/// [^note]: We have to manage our own fifo implementation because to read a buffer, we know the size to set _after_ getting the 2 slices ().
pub(crate) struct Fifo {
    /// An simple array
    ptr: *mut u8, //TODO: Is "*mut u8" the best option ?
    /// The capacity of this array
    capacity: usize,
    /// Position where datas begin in this array
    beg: usize,
    /// The size feeded
    sz_feeded: usize,
}

impl Fifo {
    ///
    /// Create a new FIFO
    pub fn new(capacity: usize) -> Result<Self> {
        if capacity == 0 {
            return Err("Allocation of size zero is not allowed".into()).into();
        }

        let layout = Layout::array::<u8>(capacity)?;

        Ok(Self {
            beg: 0,
            sz_feeded: 0,
            ptr: unsafe { alloc_zeroed(layout) }, // Because not zeroed this make this function a cancer :-)
            capacity,
        })
    }
    ///
    /// Change the capacity
    pub fn set_capacity(&mut self, new_capacity: usize) -> Result<()> {
        // New size must not erase data
        if new_capacity < self.sz_feeded {
            return Err("Attempt to resize less than feeded size".into()).into();
        }

        //    If size is a decrease :
        //      If the array will be truncate :
        //          Displace the right slice to the left, and update the begin position
        //          Set new size
        //    If size is increase :
        //      If there's a left slice :
        //          Set new size
        //          Displace the right slice to the right, and update the begin position
        //  - Decrease size :
        //     case 1a :   [. . 1 2 3 4 5 .^. .] => [. . 1 2 3 4 5 .]<del>] <-- nothing to move
        //     case 1b :   [. . . . . 1 2 3^4 5] => [. . . 1 2 3 4 5]<del>] <-- move not overlapped
        //  - Increase size :
        //     case 2a :   [. . 1 2 3 4 5 .]<none>] => [. . 1 2 3 4 5 .^. .] <-- nothing to move
        //     case 2b :   [4 5 . . . 1 2 3]<none>] => [4 5 . . . . . 1 2 3] <-- (less bytes to move left slice) move not overlapped
        //
        //TODO: Manage this cases more , I have to verify if it's really fastest:
        //  - Decrease size :
        //     case 1a :   [. . 1 2 3 4 5 .^. .] => [. . 1 2 3 4 5 .]<del>] <-- nothing to move
        //     case 1b :   [. . . . . 1 2 3^4 5] => [4 5 . . . 1 2 3]<del>] <-- move not overlapped
        //     case 1c :   [5 . . . . . 1 2^3 4] => [5 . . . 1 2 3 4]<del>] <-- move overlapped
        //            we also could do : [3 4 5 . . . 1 2]<del>] but who can say which one is the fastest ?
        //  - Increase size :
        //     case 2a :   [. . 1 2 3 4 5 .]<none>] => [. . 1 2 3 4 5 .^. .] <-- nothing to move
        //     case 2b :   [4 5 . . . 1 2 3]<none>] => [4 5 . . . . . 1^2 3] <-- move overlapped
        //     case 2c1:   [3 4 5 . . . 1 2]<none>] => [3 4 5 . . . . .^1 2] <-- (less bytes to move right size) move not overlapped
        //     case 2c2:   [4 5 . . . 1 2 3]<none>] => [. . . . . 1 2 3 4 5] <-- (less bytes to move left slice) move not overlapped
        if new_capacity == self.capacity {
            // Same size so nothing to do !
        } else if new_capacity < self.capacity {
            // Ok, capacity decrease
            if new_capacity < self.beg + self.sz_feeded {
                // Have to displace slice1
                let sz_slice1 = cmp::min(self.capacity - self.beg, self.sz_feeded);
                let new_beg_slice1 = new_capacity - sz_slice1;
                if new_capacity <= self.beg {
                    // Non overlapped if new_pos_max (so len-1) < old_beg
                    unsafe {
                        std::ptr::copy_nonoverlapping(
                            self.ptr.add(self.beg),
                            self.ptr.add(new_beg_slice1),
                            sz_slice1,
                        )
                    };
                } else {
                    // Overlapped
                    unsafe {
                        std::ptr::copy(
                            self.ptr.add(self.beg),
                            self.ptr.add(new_beg_slice1),
                            sz_slice1,
                        )
                    };
                }
                self.beg = new_beg_slice1;
            }
            self.resize_ptr(new_capacity)?; //TODO: Cancel moves that preceding case of error allocation
        } else {
            // Increase (must be before moving datas, to avoid writing over the current memory)
            self.resize_ptr(new_capacity)?;

            if self.capacity < self.beg + self.sz_feeded {
                // Have to displace slice1
                let sz_slice1 = cmp::min(self.capacity - self.beg, self.sz_feeded);
                let new_beg_slice1 = new_capacity - sz_slice1;
                if new_beg_slice1 >= self.capacity {
                    // Non overlapped if old_pos_max (so len-1) < new_beg
                    unsafe {
                        std::ptr::copy_nonoverlapping(
                            self.ptr.add(self.beg),
                            self.ptr.add(new_beg_slice1),
                            sz_slice1,
                        )
                    };
                } else {
                    // Overlapped
                    unsafe {
                        std::ptr::copy(
                            self.ptr.add(self.beg),
                            self.ptr.add(new_beg_slice1),
                            sz_slice1,
                        )
                    };
                }
                self.beg = new_beg_slice1;
            }
        }
        self.capacity = new_capacity;

        Ok(())
    }
    ///
    /// Internal function to resize the ptr of the internal array
    fn resize_ptr(&mut self, new_capacity: usize) -> Result<()> {
        let layout = match Layout::array::<u8>(new_capacity) {
            Ok(l) => l,
            Err(err) => {
                // It's preferable to panic here because some data
                // may have changed according to this reallocation, so
                // any reused have to be avoid !
                panic!("{err:?}");
            }
        };
        let tmp_ptr = unsafe { realloc(self.ptr, layout, new_capacity) };
        if false == tmp_ptr.is_null() {
            self.ptr = tmp_ptr;
        }
        Ok(())
    }
    ///
    /// Return the position of the virtual free space
    fn pos_free(&self) -> usize {
        let pos_over = self.beg + self.sz_feeded;
        if self.capacity <= pos_over {
            pos_over - self.capacity
        } else {
            pos_over
        }
    }
    ///
    /// Return the size of the virtual free space
    fn sz_free(&self) -> usize {
        self.capacity - self.sz_feeded
    }
    ///
    /// Calculate the sizes of the 2Slices
    fn sz_2slices(&self, beg: usize, len: usize) -> (usize, usize) {
        let pos_over = beg + len;
        if pos_over <= self.capacity {
            (len, 0)
        } else {
            let sz_1 = self.capacity - beg;
            let sz_2 = pos_over - self.capacity;
            (sz_1, sz_2)
        }
    }
    ///
    /// Size of the occupied space
    pub fn len(&self) -> usize {
        self.sz_feeded
    }
    ///
    /// Return a 2slices on free space. The `commit` function has to be called after this one.
    /// ! Those slices are for write usage only !
    pub fn get_available_mut(&self) -> (&mut [u8], &mut [u8]) {
        // Take the maximum it can take
        let sz_to_use = self.sz_free();
        // Get the positions of the part
        let beg = self.pos_free();
        let (sz_1, sz_2) = self.sz_2slices(beg, sz_to_use);
        // Convert to slices
        let slice_1 = unsafe { std::slice::from_raw_parts_mut(self.ptr.add(beg), sz_1) };
        let slice_2 = unsafe { std::slice::from_raw_parts_mut(self.ptr, sz_2) };
        // Return
        (slice_1, slice_2)
    }
    ///
    /// Return a 2slices on data feeded
    /// ! Those slices are for read usage only !
    pub fn get_feeded_mut(&self) -> (&mut [u8], &mut [u8]) {
        // Take the maximum it can take
        let sz_to_use = self.sz_feeded;
        // Get the positions of the part
        let beg = self.beg;
        let (sz_1, sz_2) = self.sz_2slices(beg, sz_to_use);
        // Convert to slices
        let slice_1 = unsafe { std::slice::from_raw_parts_mut(self.ptr.add(beg), sz_1) };
        let slice_2 = unsafe { std::slice::from_raw_parts_mut(self.ptr, sz_2) };
        // Return
        (slice_1, slice_2)
    }
    ///
    /// Commit data in the free space as feeded
    pub fn commit(&mut self, sz: usize) {
        self.sz_feeded += sz;
        if cfg!(debug_assertions) {
            if self.sz_feeded > self.capacity {
                panic!(
                    "self.sz_feeded > self.capacity ! (left={} right={})",
                    self.sz_feeded, self.capacity
                );
            }
        }
    }
    ///
    /// Virtually free memory and return the 2Slices of this space
    /// - If there's not enough space for the size, only the maximum available is return
    /// - The values of thoses slices have to be read just after the call of this function
    /// - Those slices are for read usage only !
    pub fn pop(&mut self, sz: usize) -> (&mut [u8], &mut [u8]) {
        // Can't remove more than available
        let sz_to_use = cmp::min(sz, self.sz_feeded);
        //let pos_array = self.beg;
        // Get the positions of the part
        let (sz_1, sz_2) = self.sz_2slices(self.beg, sz_to_use);
        // Convert to slices
        let slice_1 = unsafe { std::slice::from_raw_parts_mut(self.ptr.add(self.beg), sz_1) };
        let slice_2 = unsafe { std::slice::from_raw_parts_mut(self.ptr, sz_2) };
        // Virtually free
        if sz_2 > 0 {
            self.beg = sz_2;
        } else {
            if self.beg + sz_1 == self.capacity {
                self.beg = 0;
            } else {
                self.beg += sz_1;
            }
        }
        self.sz_feeded -= sz_to_use;
        // Return
        (slice_1, slice_2)
    }
    ///
    /// For debug purpose only !
    fn is_feeded(&self, pos: usize) -> bool {
        let (sz_1, sz_2) = self.sz_2slices(self.beg, self.sz_feeded);
        if pos >= self.beg && pos < self.beg + sz_1 {
            return true;
        }
        if pos < sz_2 {
            return true;
        }
        return false;
    }
}
///
/// Defining a drop function is the price to pay for unsafe allocation
impl Drop for Fifo {
    fn drop(&mut self) {
        let layout = Layout::array::<u8>(self.capacity).expect("Desallocation error!!");
        unsafe { dealloc(self.ptr, layout) };
    }
}
///
/// Debug part
impl Debug for Fifo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let empty = "NaN".to_string();
        let mut dbg = f.debug_tuple("");
        for i in 0..self.capacity {
            if self.is_feeded(i) {
                let v = unsafe { *self.ptr.add(i) as u8 };
                if i == self.beg {
                    dbg.field(&format!("<"));
                }
                dbg.field(&v);
            } else {
                dbg.field(&empty);
            }
        }
        let (s1, s2) = self.get_feeded_mut();

        dbg.field(&format!(
            "len={} str=[{}^{}]",
            self.len(),
            &String::from_utf8_lossy(s1),
            &String::from_utf8_lossy(s2)
        ));

        dbg.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_common() {
        let mut fifo = Fifo::new(5).unwrap();
        {
            let (sl1, _sl2) = fifo.get_available_mut();
            sl1[0..5].copy_from_slice(&[1u8, 2, 3, 4, 5]);
            fifo.commit(5);
            // Excepted = [1 2 3 4 5]
            let (sl1, sl2) = fifo.get_feeded_mut();
            assert_eq!(sl1, &[1u8, 2, 3, 4, 5], "Case 1 : for sl1");
            assert_eq!(sl2, &[], "Case 1 : for sl2");
        }
        {
            fifo.pop(2);
            // Excepted = [. .<3 4 5]
            let (sl1, sl2) = fifo.get_feeded_mut();
            assert_eq!(sl1, &[3u8, 4, 5], "Case 2 : for sl1");
            assert_eq!(sl2, &[], "Case 2 : for sl2");
        }
        {
            let (sl1, sl2) = fifo.get_available_mut();
            assert_eq!(sl1.len(), 2, "Case 3 : for sl1.len()");
            assert_eq!(sl2.len(), 0, "Case 3 : for sl2.len()");
            sl1[0..2].copy_from_slice(&[6u8, 7]);
            fifo.commit(2);
            // Excepted = [6 7<3 4 5]
            let (sl1, sl2) = fifo.get_feeded_mut();
            assert_eq!(sl1, &[3u8, 4, 5], "Case 3 : for sl1");
            assert_eq!(sl2, &[6u8, 7], "Case 3 : for sl2");
        }
        {
            let _ = fifo.set_capacity(8);
            // Excepted = from [6 7<3 4 5] to [6 7 . . .<3 4 5]
            let (sl1, sl2) = fifo.get_feeded_mut();
            assert_eq!(sl1, &[3u8, 4, 5], "Case 4 : for sl1");
            assert_eq!(sl2, &[6u8, 7], "Case 4 : for sl2");
        }
        {
            let (sl1, _sl2) = fifo.get_available_mut();
            sl1[0..3].copy_from_slice(&[8u8, 9, 10]);
            fifo.commit(3);
            // Excepted = [6 7 8 9 10<3 4 5]
            let (sl1, sl2) = fifo.get_feeded_mut();
            assert_eq!(sl1, &[3u8, 4, 5], "Case 5 : for sl1");
            assert_eq!(sl2, &[6u8, 7, 8, 9, 10], "Case 5 : for sl2");
        }
        {
            let (pop1, pop2) = fifo.pop(2);
            // Excepted = from [6 7 8 9 10<3 4 5] to [6 7 8 9 10 . .<5]
            assert_eq!(pop1, &[3u8, 4], "Case 6 : for pop1");
            assert_eq!(pop2, &[], "Case 6 : for pop2");
            let (sl1, sl2) = fifo.get_feeded_mut();
            assert_eq!(sl1, &[5u8], "Case 6 : for sl1");
            assert_eq!(sl2, &[6u8, 7, 8, 9, 10], "Case 6 : for sl2");
        }
        {
            let _ = fifo.set_capacity(7);
            // Excepted = from [6 7 8 9 10 .<5] (decrease capacity, no overlap)
            let (sl1, sl2) = fifo.get_feeded_mut();
            assert_eq!(sl1, &[5u8], "Case 7 : for sl1");
            assert_eq!(sl2, &[6u8, 7, 8, 9, 10], "Case 7 : for sl2");
        }
        {
            let _ = fifo.set_capacity(6);
            // Excepted = [6 7 8 9 10 <5] (decrease capacity, no overlap)
            let (sl1, sl2) = fifo.get_feeded_mut();
            assert_eq!(sl1, &[5u8], "Case 8 : for sl1");
            assert_eq!(sl2, &[6u8, 7, 8, 9, 10], "Case 8 : for sl2");
        }
        {
            let (pop1, pop2) = fifo.pop(2);
            // Excepted = from [6 7 8 9 10 <5] to [.<7 8 9 10 .]
            assert_eq!(pop1, &[5u8], "Case 9 : for pop1");
            assert_eq!(pop2, &[6u8], "Case 9 : for pop2");
            let (sl1, sl2) = fifo.get_feeded_mut();
            assert_eq!(sl1, &[7u8, 8, 9, 10], "Case 9 : for sl1");
            assert_eq!(sl2, &[], "Case 9 : for sl2");
        }
        {
            let _ = fifo.set_capacity(4);
            // Excepted = [.<7 8 9 10 .] to [<7 8 9 10 ]
            //*println!("case 10 / fifo=[{fifo:?}]");
            let (sl1, sl2) = fifo.get_feeded_mut();
            assert_eq!(sl1, &[7u8, 8, 9, 10], "Case 10 : for sl1");
            assert_eq!(sl2, &[], "Case 10 : for sl2");
        }
        {
            let (_pop1, _pop2) = fifo.pop(2);
            // Excepted = [<7 8 9 10] to [. .<9 10]
            //*println!("case 11 / fifo=[{fifo:?}]");
            let (sl1, sl2) = fifo.get_feeded_mut();
            assert_eq!(sl1, &[9u8, 10], "Case 11 : for sl1");
            assert_eq!(sl2, &[], "Case 11 : for sl2");
        }
        {
            let _ = fifo.set_capacity(5);
            // Excepted = [. .<9 10] to [. .<9 10 .]
            //*println!("case 12 / fifo=[{fifo:?}]");
            let (sl1, sl2) = fifo.get_feeded_mut();
            assert_eq!(sl1, &[9u8, 10], "Case 12 : for sl1");
            assert_eq!(sl2, &[], "Case 12 : for sl2");
        }
        {
            let (sl1, sl2) = fifo.get_available_mut();
            sl1[0..1].copy_from_slice(&[11u8]);
            sl2[0..1].copy_from_slice(&[12u8]);
            fifo.commit(2);
            // Excepted = [. .<9 10 .] to [12 .<9 10 11]
            let (sl1, sl2) = fifo.get_feeded_mut();
            assert_eq!(sl1, &[9u8, 10, 11], "Case 13 : for sl1");
            assert_eq!(sl2, &[12u8], "Case 13 : for sl2");
        }
        {
            let _ = fifo.set_capacity(6);
            // Excepted = [12 .<9 10 11] to [12 . .<9 10 11] (increase capacity with copy overlapped)
            let (sl1, sl2) = fifo.get_feeded_mut();
            assert_eq!(sl1, &[9u8, 10, 11], "Case 14 : for sl1");
            assert_eq!(sl2, &[12u8], "Case 14 : for sl2");
        }
        {
            let _ = fifo.set_capacity(5);
            // Excepted = [12 . .<9 10 11] to [12 .<9 10 11] (decrease capacity with copy overlapped)
            let (sl1, sl2) = fifo.get_feeded_mut();
            assert_eq!(sl1, &[9u8, 10, 11], "Case 15 : for sl1");
            assert_eq!(sl2, &[12u8], "Case 15 : for sl2");
        }
    }
}
