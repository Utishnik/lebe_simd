// TODO: SIMD
use crate::call_single_arg_macro_for_each;
#[cfg(feature = "simd")]
use crate::io::bytes;
use crate::io::ReadEndian;
use crate::io::WriteEndian;
use crate::Endian;
use std::io::{Read, Result, Write};
macro_rules! implement_simple_primitive_write {
    ($type: ident) => {
        impl<R: Read> ReadEndian<[$type]> for R {
            fn read_from_little_endian_into(&mut self, value: &mut [$type]) -> Result<()> {
                unsafe { bytes::read_slice(self, value)? };
                value.convert_little_endian_to_current();
                Ok(())
            }

            fn read_from_big_endian_into(&mut self, value: &mut [$type]) -> Result<()> {
                unsafe { bytes::read_slice(self, value)? };
                value.convert_big_endian_to_current();
                Ok(())
            }
        }
    };
}

call_single_arg_macro_for_each! {
    implement_simple_primitive_write,
    u8, u16, u32, u64, u128,
    i8, i16, i32, i64, i128,
    f64, f32
}

macro_rules! implement_slice_io {
    ($type: ident) => {
        impl<W: Write> WriteEndian<[$type]> for W {
            fn write_as_big_endian(&mut self, value: &[$type]) -> Result<()> {
                if cfg!(target_endian = "little") {
                    // FIX ME this SIMD optimization makes no difference ... why? like, ZERO difference, not even worse
                    //                #[cfg(feature = "simd")]
                    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
                    unsafe {
                        if is_x86_feature_detected!("avx2") {
                            write_bytes_avx(self, value);
                            return Ok(());
                        }
                    }

                    // otherwise (no avx2 available)
                    //                for number in value {
                    //                    self.write_as_little_endian(number);
                    //                }
                    //
                    //                return Ok(());
                    unimplemented!();

                    #[target_feature(enable = "avx2")]
                    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
                    unsafe fn write_bytes_avx(
                        write: &mut impl Write,
                        slice: &[$type],
                    ) -> Result<()> {
                        #[cfg(target_arch = "x86")]
                        use std::arch::x86 as mm;
                        #[cfg(target_arch = "x86_64")]
                        use std::arch::x86_64 as mm;

                        let bytes: &[u8] = crate::io::bytes::slice_as_bytes(slice);
                        let mut chunks = bytes.chunks_exact(32);

                        let indices = mm::_mm256_set_epi8(
                            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5,
                            6, 7, 8, 9, 10, 11, 12, 13, 14,
                            15, //                        3,2,1,0, 7,6,5,4, 11,10,9,8, 15,14,13,12,
                                //                        3,2,1,0, 7,6,5,4, 11,10,9,8, 15,14,13,12
                        );

                        for chunk in &mut chunks {
                            let data = mm::_mm256_loadu_si256(chunk.as_ptr() as _);
                            let result = mm::_mm256_shuffle_epi8(data, indices);
                            let mut out = [0_u8; 32];
                            mm::_mm256_storeu_si256(out.as_mut_ptr() as _, result);
                            write.write_all(&out)?;
                        }

                        let remainder = chunks.remainder();

                        {
                            // copy remainder into larger slice, with zeroes at the end
                            let mut last_chunk = [0_u8; 32];
                            last_chunk[0..remainder.len()].copy_from_slice(remainder);
                            let data = mm::_mm256_loadu_si256(last_chunk.as_ptr() as _);
                            let result = mm::_mm256_shuffle_epi8(data, indices);
                            mm::_mm256_storeu_si256(last_chunk.as_mut_ptr() as _, result);
                            write.write_all(&last_chunk[0..remainder.len()])?;
                        }

                        Ok(())
                    }
                } else {
                    unsafe {
                        bytes::write_slice(self, value)?;
                    }
                    Ok(())
                }
            }

            fn write_as_little_endian(&mut self, value: &[$type]) -> Result<()> {
                for number in value {
                    //self.write_as_little_endian(number)?; todo
                }

                Ok(())
            }
        }
    };
}

call_single_arg_macro_for_each! {
    implement_slice_io,
    u8, u16, u32, u64, u128,
    i8, i16, i32, i64, i128,
    f64, f32
}
