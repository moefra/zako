use shadow_rs::ShadowBuilder;
use shadow_rs::BuildPattern;

fn main() {
    ShadowBuilder::builder()
        .build_pattern(BuildPattern::RealTime)
        .build().unwrap();
}
