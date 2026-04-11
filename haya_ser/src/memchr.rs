fn memchr_fallback(needle: u8, mut ptr: *const u8, end: *const u8) -> *const u8 {
    unsafe {
        while !core::ptr::eq(ptr, end) {
            let ch = *ptr;
            if ch == needle {
                break;
            }
            ptr = ptr.add(1);
        }
    }
    ptr
}

unsafe fn memchr2_fallback(
    needle1: u8,
    needle2: u8,
    mut ptr: *const u8,
    end: *const u8,
) -> *const u8 {
    unsafe {
        while !core::ptr::eq(ptr, end) {
            let ch = *ptr;
            if ch == needle1 || ch == needle2 {
                break;
            }
            ptr = ptr.add(1);
        }
    }
    ptr
}

unsafe fn memchr3_fallback(
    needle1: u8,
    needle2: u8,
    needle3: u8,
    mut ptr: *const u8,
    end: *const u8,
) -> *const u8 {
    unsafe {
        while !core::ptr::eq(ptr, end) {
            let ch = *ptr;
            if ch == needle1 || ch == needle2 || ch == needle3 {
                break;
            }
            ptr = ptr.add(1);
        }
    }
    ptr
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[cfg(target_feature = "avx2")]
pub(crate) fn memchr(needle: u8, mut ptr: *const u8, end: *const u8) -> *const u8 {
    #[cfg(target_arch = "x86")]
    use core::arch::x86::{
        __m256i, _mm256_cmpeq_epi8, _mm256_loadu_si256, _mm256_movemask_epi8, _mm256_set1_epi8,
    };
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{
        __m256i, _mm256_cmpeq_epi8, _mm256_loadu_si256, _mm256_movemask_epi8, _mm256_set1_epi8,
    };
    unsafe {
        // let offset = ptr.align_offset(32);
        // if offset > 0 {
        //     let end = ptr.add(offset).min(end);
        //     while ptr != end {
        //         if *ptr == needle {
        //             return ptr;
        //         }
        //         ptr = ptr.add(1);
        //     }
        // }
        let n = _mm256_set1_epi8(needle.cast_signed());
        while ptr.add(32) <= end {
            let v = _mm256_loadu_si256(ptr.cast::<__m256i>());
            let m = _mm256_movemask_epi8(_mm256_cmpeq_epi8(v, n)).cast_unsigned();
            if m != 0 {
                return ptr.add(m.trailing_zeros() as usize);
            }
            ptr = ptr.add(32);
        }

        memchr_fallback(needle, ptr, end)
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[cfg(not(target_feature = "avx2"))]
pub(crate) fn memchr(needle: u8, ptr: *const u8, end: *const u8) -> *const u8 {
    memchr_fallback(needle, ptr, end)
}

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
pub(crate) fn memchr(needle: u8, ptr: *const u8, end: *const u8) -> *const u8 {
    memchr_fallback(needle, ptr, end)
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[cfg(target_feature = "avx2")]
pub(crate) fn memchr2(needle1: u8, needle2: u8, mut ptr: *const u8, end: *const u8) -> *const u8 {
    #[cfg(target_arch = "x86")]
    use core::arch::x86::{
        __m256i, _mm256_cmpeq_epi8, _mm256_loadu_si256, _mm256_movemask_epi8, _mm256_or_si256,
        _mm256_set1_epi8,
    };
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{
        __m256i, _mm256_cmpeq_epi8, _mm256_loadu_si256, _mm256_movemask_epi8, _mm256_or_si256,
        _mm256_set1_epi8,
    };
    unsafe {
        // let offset = ptr.align_offset(32);
        // if offset > 0 {
        //     let end = ptr.add(offset).min(end);
        //     while ptr != end {
        //         if *ptr == needle1 || *ptr == needle2 {
        //             return ptr;
        //         }
        //         ptr = ptr.add(1);
        //     }
        // }
        let n1 = _mm256_set1_epi8(needle1.cast_signed());
        let n2 = _mm256_set1_epi8(needle2.cast_signed());
        while ptr.add(32) <= end {
            let v = _mm256_loadu_si256(ptr.cast::<__m256i>());
            let m = _mm256_movemask_epi8(_mm256_or_si256(
                _mm256_cmpeq_epi8(v, n1),
                _mm256_cmpeq_epi8(v, n2),
            ))
            .cast_unsigned();
            if m != 0 {
                return ptr.add(m.trailing_zeros() as usize);
            }
            ptr = ptr.add(32);
        }

        memchr2_fallback(needle1, needle2, ptr, end)
    }
}

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
pub(crate) fn memchr2(needle1: u8, needle2: u8, ptr: *const u8, end: *const u8) -> *const u8 {
    memchr2_fallback(needle1, needle2, ptr, end)
}

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
pub(crate) fn memchr2(needle1: u8, needle2: u8, ptr: *const u8, end: *const u8) -> *const u8 {
    memchr2_fallback(needle1, needle2, ptr, end)
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[cfg(target_feature = "avx2")]
pub(crate) fn memchr3(
    needle1: u8,
    needle2: u8,
    needle3: u8,
    mut ptr: *const u8,
    end: *const u8,
) -> *const u8 {
    #[cfg(target_arch = "x86")]
    use core::arch::x86::{
        __m256i, _mm256_cmpeq_epi8, _mm256_loadu_si256, _mm256_movemask_epi8, _mm256_or_si256,
        _mm256_set1_epi8,
    };
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{
        __m256i, _mm256_cmpeq_epi8, _mm256_loadu_si256, _mm256_movemask_epi8, _mm256_or_si256,
        _mm256_set1_epi8,
    };
    unsafe {
        // let offset = ptr.align_offset(32);
        // if offset > 0 {
        //     let end = ptr.add(offset).min(end);
        //     while ptr != end {
        //         if *ptr == needle1 || *ptr == needle2 || *ptr == needle3 {
        //             return ptr;
        //         }
        //         ptr = ptr.add(1);
        //     }
        // }
        let n1 = _mm256_set1_epi8(needle1.cast_signed());
        let n2 = _mm256_set1_epi8(needle2.cast_signed());
        let n3 = _mm256_set1_epi8(needle3.cast_signed());
        while ptr.add(32) <= end {
            let v = _mm256_loadu_si256(ptr.cast::<__m256i>());
            let m = _mm256_movemask_epi8(_mm256_or_si256(
                _mm256_or_si256(_mm256_cmpeq_epi8(v, n1), _mm256_cmpeq_epi8(v, n2)),
                _mm256_cmpeq_epi8(v, n3),
            ))
            .cast_unsigned();
            if m != 0 {
                return ptr.add(m.trailing_zeros() as usize);
            }
            ptr = ptr.add(32);
        }

        memchr3_fallback(needle1, needle2, needle3, ptr, end)
    }
}

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
pub(crate) fn memchr3(
    needle1: u8,
    needle2: u8,
    needle3: u8,
    ptr: *const u8,
    end: *const u8,
) -> *const u8 {
    memchr3_fallback(needle1, needle2, needle3, ptr, end)
}

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
pub(crate) fn memchr3(
    needle1: u8,
    needle2: u8,
    needle3: u8,
    ptr: *const u8,
    end: *const u8,
) -> *const u8 {
    memchr3_fallback(needle1, needle2, needle3, ptr, end)
}
