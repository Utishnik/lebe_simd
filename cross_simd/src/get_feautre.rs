//todo arm neon
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod x86_backend {
    #[derive(PartialEq)]
    pub enum Feature {
        Avx,
        Sse,
        SseOld,
    }
    cpufeatures::new!(cpuid_old_sse, "sse2");
    cpufeatures::new!(cpuid_sse, "sse4.2");
    cpufeatures::new!(cpuid_avx, "avx");
    /*avx2 и тд мой проц не потдержует( так чо тестить не могу*/

    pub fn check_old_sse() -> bool {
        let token: cpuid_old_sse::InitToken = cpuid_old_sse::init();
        token.get()
    }

    pub fn check_sse() -> bool {
        let token: cpuid_sse::InitToken = cpuid_sse::init();
        token.get()
    }

    pub fn check_avx() -> bool {
        let token: cpuid_avx::InitToken = cpuid_avx::init();
        token.get()
    }

    pub fn checking_to_reduction() -> Option<Feature> {
        let avx: bool = check_avx();
        if avx {
            return Some(Feature::Avx);
        }

        let sse: bool = check_sse();
        if sse {
            return Some(Feature::Sse);
        }

        let sse_old: bool = check_sse();
        if sse_old {
            return Some(Feature::SseOld);
        }

        None
    }
}

#[test]
fn test() {
    let res: Option<x86_backend::Feature> = x86_backend::checking_to_reduction();
    if res.unwrap() == x86_backend::Feature::Avx{
        println!("avx");
    }
}
