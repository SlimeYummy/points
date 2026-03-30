//
// simple bubble sort
//

#[inline]
pub fn bubble_sort_by<T, F>(arr: &mut [T], mut is_less: F)
where
    F: FnMut(&T, &T) -> bool,
{
    let mut swapped = true;
    while swapped {
        swapped = false;
        for i in 1..arr.len() {
            if is_less(&arr[i], &arr[i - 1]) {
                arr.swap(i, i - 1);
                swapped = true;
            }
        }
    }
}

#[inline]
pub fn bubble_sort<T>(arr: &mut [T])
where
    T: Ord,
{
    bubble_sort_by(arr, |a, b| a < b);
}

//
// find in slice with an initial offset
//

#[inline]
pub fn index_offset_by<'t, T, F>(slice: &'t [T], offset: usize, pred: F) -> Option<usize>
where
    F: Fn(&T) -> bool,
{
    for idx in 0..slice.len() {
        let pos = (offset + idx) % slice.len();
        if pred(&slice[pos]) {
            return Some(pos);
        }
    }
    None
}

#[inline]
pub fn find_offset_by<'t, T, F>(slice: &'t [T], offset: usize, pred: F) -> Option<&'t T>
where
    F: Fn(&T) -> bool,
{
    index_offset_by(slice, offset, pred).map(|pos| &slice[pos])
}

#[inline]
pub fn find_mut_offset_by<'t, T, F>(slice: &'t mut [T], offset: usize, pred: F) -> Option<&'t mut T>
where
    F: Fn(&T) -> bool,
{
    index_offset_by(slice, offset, pred).map(|pos| &mut slice[pos])
}

#[inline]
pub fn index_offset<'t, T: PartialEq>(slice: &'t [T], offset: usize, value: &T) -> Option<usize> {
    index_offset_by(slice, offset, |v| v == value)
}

#[inline]
pub fn find_offset<'t, T: PartialEq>(slice: &'t [T], offset: usize, value: &T) -> Option<&'t T> {
    find_offset_by(slice, offset, |v| v == value)
}

#[inline]
pub fn find_mut_offset<'t, T: PartialEq>(slice: &'t mut [T], offset: usize, value: &T) -> Option<&'t mut T> {
    find_mut_offset_by(slice, offset, |v| v == value)
}
