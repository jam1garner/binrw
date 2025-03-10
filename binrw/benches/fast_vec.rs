#![cfg_attr(nightly, feature(test))]

#[path = "../tests/test_helpers/mod.rs"]
mod test_helpers;

#[cfg(nightly)]
mod fast_vec {
    extern crate test;

    use super::test_helpers;
    use binrw::BinRead;

    #[cfg(target_endian = "big")]
    const ENDIAN: binrw::Endian = binrw::Endian::Little;
    #[cfg(target_endian = "little")]
    const ENDIAN: binrw::Endian = binrw::Endian::Big;

    macro_rules! bench_fast {
        ($($name:ident, $Ty:ty, $reads:literal);+$(;)?) => {
            $(#[bench]
            fn $name(b: &mut test::Bencher) {
                run_benchmark::<$Ty>(b, binrw::Endian::NATIVE, $reads);
            })+
        }
    }

    macro_rules! bench_fast_swap {
        ($($name:ident, $Ty:ty, $reads:literal);+$(;)?) => {
            $(#[bench]
            fn $name(b: &mut test::Bencher) {
                run_benchmark::<$Ty>(b, ENDIAN, $reads);
            })+
        }
    }

    #[inline(always)]
    fn run_benchmark<T>(b: &mut test::Bencher, endian: binrw::Endian, expected_reads: usize)
    where
        T: for<'a> BinRead<Args<'a> = ()> + 'static,
    {
        let mut filler = test_helpers::Fill::new(1);
        let count: usize = 65536 / core::mem::size_of::<T>();

        b.iter(|| {
            let mut counter = test_helpers::Counter::new(&mut filler);

            test::black_box(
                Vec::<T>::read_options(
                    &mut counter,
                    endian,
                    binrw::VecArgs::builder().count(count).finalize(),
                )
                .unwrap(),
            );

            assert_eq!(counter.reads, expected_reads);
        });
    }

    bench_fast!(
        vec_i8, i8, 12;
        vec_u8, u8, 4;
        vec_i16, i16, 12;
        vec_u16, u16, 12;
        vec_i32, i32, 12;
        vec_u32, u32, 12;
        vec_i64, i64, 12;
        vec_u64, u64, 12;
        vec_i128, i128, 11;
        vec_u128, u128, 11;
    );

    bench_fast_swap!(
        vec_i16_swap, i16, 12;
        vec_u16_swap, u16, 12;
        vec_i32_swap, i32, 12;
        vec_u32_swap, u32, 12;
        vec_i64_swap, i64, 12;
        vec_u64_swap, u64, 12;
        vec_i128_swap, i128, 11;
        vec_u128_swap, u128, 11;
    );
}
