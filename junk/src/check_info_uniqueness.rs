use std::{collections::BTreeMap, fmt::Debug, hash::Hash};

use bytes::Bytes;
use itertools::Itertools;
use owo_colors::OwoColorize;

fn check_unique_by<
    T: Debug + PartialEq + Eq + Hash,
    K: Debug + Clone + PartialOrd + Ord + PartialEq + Eq + Hash,
    F: Fn(&T) -> K,
>(
    what: &str,
    list: &[T],
    f: F,
) {
    let mut duplicates = BTreeMap::<_, Vec<_>>::new();

    for item in list.iter().unique() {
        let values = duplicates.entry(f(item)).or_default();
        values.push(item);
    }

    let duplicate_count = duplicates.values().filter(|x| x.len() > 1).count();

    if duplicate_count > 0 {
        println!("{} {}", what.blue().bold(), "is not unique!".red());
        println!("  {} duplicates:", duplicate_count);
        for (key, values) in duplicates {
            if values.len() > 1 {
                println!("    {:?}: {:?}", key, values);
            }
        }
    } else {
        println!("{} {}", what.blue().bold(), "is unique!".green());
    }
}

pub fn main(snr_path: String) {
    let scenario = std::fs::read(snr_path).unwrap();
    let scenario = Bytes::from(scenario);
    let scenario = shin_core::format::scenario::Scenario::new(scenario).unwrap();
    let info = scenario.info_tables();

    check_unique_by("mask_info", &info.mask_info, |x| x.name.0.clone());
    check_unique_by("picture_info", &info.picture_info, |x| x.name.0.clone());

    // this is __mostly__ unique, just requires a lipsync character id for disambiguation b/w chars 27 and 60. meh, make it optional or smth
    check_unique_by("bustup_info", &info.bustup_info, |x| {
        (
            x.name.0.clone(),
            x.emotion.0.clone(),
            x.lipsync_character_id,
        )
    });

    // mostly unique, requires a disambiguation for ¿debug menu bgm?
    check_unique_by("bgm_info", &info.bgm_info, |x| x.name.0.clone());
    check_unique_by("se_info", &info.se_info, |x| x.name.0.clone());
    check_unique_by("movie_info", &info.movie_info, |x| x.name.0.clone());
    check_unique_by("picture_box_info", &info.picture_box_info, |x| {
        x.name.0.clone()
    });
    check_unique_by("music_box_info", &info.music_box_info, |x| x.bgm_id);
}
