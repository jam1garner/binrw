//! container module
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

/// Container
pub trait Container: Sized + IntoIterator {
    /// Count
    type Count;

    /// naive
    ///
    /// # Errors
    ///
    /// If `f` returns an error, the error will be returned
    fn new_naive<Fun, Error>(count: Self::Count, f: Fun) -> Result<Self, Error>
    where
        Fun: FnMut() -> Result<Self::Item, Error>;

    /// smart
    ///
    /// # Errors
    ///
    /// If `f` returns an error, the error will be returned
    fn new_smart<Fun, Error>(count: Self::Count, f: Fun) -> Result<Self, Error>
    where
        Fun: FnMut(&mut [Self::Item]) -> Result<(), Error>,
        Self::Item: Default + Clone;

    // whee it's a Functor
    /// Type Constructor
    type HigherSelf<T>: Container<Count = Self::Count, HigherSelf<Self::Item> = Self>
        + IntoIterator<Item = T>;

    /// map
    fn map<Fun, T>(self, f: Fun) -> Self::HigherSelf<T>
    where
        Fun: FnMut(Self::Item) -> T;
}

impl<T, const N: usize> Container for [T; N] {
    type Count = ();
    type HigherSelf<X> = [X; N];

    fn new_naive<Fun, Error>(_count: (), mut f: Fun) -> Result<Self, Error>
    where
        Fun: FnMut() -> Result<Self::Item, Error>,
    {
        array_init::try_array_init(|_| f())
    }

    fn new_smart<Fun, Error>(_count: (), mut f: Fun) -> Result<Self, Error>
    where
        Fun: FnMut(&mut [Self::Item]) -> Result<(), Error>,
        Self::Item: Default + Clone,
    {
        let mut res = array_init::array_init(|_| Self::Item::default());
        f(&mut res)?;
        Ok(res)
    }

    fn map<Fun, X>(self, f: Fun) -> Self::HigherSelf<X>
    where
        Fun: FnMut(Self::Item) -> X,
    {
        self.map(f)
    }
}

impl<T> Container for Vec<T> {
    type Count = usize;
    type HigherSelf<X> = Vec<X>;

    fn new_naive<Fun, Error>(count: usize, f: Fun) -> Result<Self, Error>
    where
        Fun: FnMut() -> Result<Self::Item, Error>,
    {
        core::iter::repeat_with(f).take(count).collect()
    }

    fn new_smart<Fun, Error>(count: usize, mut f: Fun) -> Result<Self, Error>
    where
        Fun: FnMut(&mut [Self::Item]) -> Result<(), Error>,
        Self::Item: Default + Clone,
    {
        let mut list = Self::default();
        let mut start = 0;
        let mut remaining = count;
        // Allocating and reading from the source in chunks is done to keep
        // a bad `count` from causing huge memory allocations that are
        // doomed to fail
        while remaining != 0 {
            // Using a similar strategy as std `default_read_to_end` to
            // leverage the memory growth strategy of the underlying Vec
            // implementation (in std this will be exponential) using a
            // minimum byte allocation
            let growth: usize = 32 / core::mem::size_of::<u32>();
            list.reserve(remaining.min(growth.max(1)));

            let items_to_read = remaining.min(list.capacity() - start);
            let end = start + items_to_read;

            // In benchmarks, this resize decreases performance by 27–40%
            // relative to using `unsafe` to write directly to uninitialised
            // memory, but nobody ever got fired for buying IBM
            list.resize(end, Self::Item::default());
            f(&mut list[start..end])?;

            remaining -= items_to_read;
            start += items_to_read;
        }

        Ok(list)
    }

    fn map<Fun, X>(self, f: Fun) -> Self::HigherSelf<X>
    where
        Fun: FnMut(Self::Item) -> X,
    {
        self.into_iter().map(f).collect()
    }
}
