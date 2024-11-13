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
