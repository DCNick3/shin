//! Tests based on single layout snapshots

use insta::assert_debug_snapshot;

use super::make_snapshot;
use crate::vm::command::types::{MessageTextLayout, MessageboxType};

#[test]
fn snapshot_ep1_nanjo1() {
    // here we have rubi text that is wider than the base text. the pointers have to be adjusted
    let snapshot = make_snapshot(MessageTextLayout::Justify, MessageboxType::Ushiromiya, "南條\u{3000}輝正@r@v19/11900001.「…………また。@k@v19/11900002.…お酒を@bたしな.@<嗜@>まれましたな？」");

    assert_debug_snapshot!("ep1_nanjo1", snapshot);
}

#[test]
fn snapshot_ep1_nanjo2() {
    // here we have a pretty long message with multiple Waits and spanning multiple lines
    let snapshot = make_snapshot(MessageTextLayout::Justify, MessageboxType::Ushiromiya, "南條\u{3000}輝正@r@v19/11900003.「…………金蔵さん。@k@v19/11900004.…あんたの体が一見調子がいいのは薬が効いてるからだ。@k@v19/11900005.だが、そんな強い酒を飲み続けては薬の意味もなくなってしまう。@k@v19/11900006.…悪いことは言わん。@k@v19/11900007.酒は控えなさい」");

    assert_debug_snapshot!("ep1_nanjo2", snapshot);
}

#[test]
fn snapshot_ep1_kinzo1() {
    // here we have rubi text that is smaller than the base text
    let snapshot = make_snapshot(MessageTextLayout::Justify, MessageboxType::Ushiromiya, "右代宮\u{3000}金蔵@r@v01/11500012.「はっはっはっは…。@k@v01/11500013.お前とて、指し間違えた手を待てと言っても聞かぬではないか。@k@v01/11500014.ならば@bあいこ.@<相子@>というものだろう」");

    assert_debug_snapshot!("ep1_kinzo1", snapshot);
}

#[test]
fn snapshot_ep1_kumasawa_nvl() {
    // tests novel mode, and, apparently, kinsoku shori-forbidden character on start of the line
    let snapshot = make_snapshot(MessageTextLayout::Justify, MessageboxType::Novel, "熊沢\u{3000}チヨ@r@v18/11800041_1.……おいたわしや、紗音さん、嘉音さん…。@k@r@v18/11800041_2.あの二人がいじめられる理由は何もないのです。@k@r@v18/11800041_3.…しかし、郷田さんに嫌われているのは紛れもない事実…。");

    assert_debug_snapshot!("ep1_kumasawa_nvl", snapshot);
}

#[test]
fn snapshot_ep2_beatrice_red_truth() {
    // tests red truth (instant text + color change), as well as section & sync commands, used to issue VM commands at certain points in the message
    let snapshot = make_snapshot(MessageTextLayout::Justify, MessageboxType::WitchSpace, "ベアトリーチェ@r@v27/20700938.「そうだ。@k@v27/20700939.だから妾はこれより、@v27/20700940.@|@y@c900.@[真実を語る時、赤を使うことにする@]@c.」");

    assert_debug_snapshot!("ep2_beatrice_red_truth", snapshot);
}

#[test]
fn snapshot_ep4_beatrice_blue_truth() {
    // tests 2 blue truths in one message
    let snapshot = make_snapshot(MessageTextLayout::Justify, MessageboxType::Novel, "ベアトリーチェ@r@v27/40700828.@|@y@c279.@[以上の復唱要求、並びに復唱拒否から、@]@k@v27/40700829.@|@y@[妾はそなたに対戦相手の資格がないことを宣言する。@]@c.");

    assert_debug_snapshot!("ep4_beatrice_blue_truth", snapshot);
}

#[test]
fn snapshot_ep1_novel_centered() {
    // test centered text in NVL mode
    let snapshot = make_snapshot(
        MessageTextLayout::Center,
        MessageboxType::Novel,
        "@r@a1666.ダカラ、@w666.現金ガスグニ大量ニ、@r@w666.喉カラ手ガ出ルホドニ欲シカッタ…！",
    );

    assert_debug_snapshot!("ep1_novel_centered", snapshot);
}

#[test]
fn snapshot_ep1_narrator1() {
    // this tests a non-novel message without a character name
    let snapshot = make_snapshot(
        MessageTextLayout::Justify,
        MessageboxType::Ushiromiya,
        "@r聴診器を外しながら、年輩の医師は溜め息を漏らす。",
    );

    assert_debug_snapshot!("ep1_narrator1", snapshot);
}

#[test]
fn snapshot_ep1_kinzo2() {
    // tests the correct implementation of multiline quoted text
    let snapshot = make_snapshot(
        MessageTextLayout::Justify,
        MessageboxType::Ushiromiya,
        "右代宮\u{3000}金蔵@r@v01/11500003.「忠告の気持ちだけはありがたくいただいておく。@k@v01/11500004.我が友よ。@k@v01/11500005.………源次。@k@v01/11500006.もう一杯頼む。@k@v01/11500007.心持ち薄めでな。@k@v01/11500008.南條の顔も立ててやれ」",
    );

    assert_debug_snapshot!("ep1_kinzo2", snapshot);
}

#[test]
fn snapshot_ep1_battler1() {
    // tests the correct handling of prohibition rules in combination with rubi text
    let snapshot = make_snapshot(
        MessageTextLayout::Justify,
        MessageboxType::Ushiromiya,
        "右代宮\u{3000}戦人@r@v10/10100014.「こんだけ@bた.@<立@>っ@bぱ.@<端@>がありゃー俺の発育は充分っすから～！@k\u{3000}@v10/10100015.むしろちょいと身長縮めた方が服が探しやすいくらい！」",
    );

    assert_debug_snapshot!("ep1_battler1", snapshot);
}

#[test]
fn snapshot_ep1_narrator2() {
    // tests the correctness of time reflowing when rubi and base texts have equal width
    let snapshot = make_snapshot(
        MessageTextLayout::Justify,
        MessageboxType::Ushiromiya,
        "@r@bねこぐるま.@<猫車@>のひとつしかない車輪が小石でも噛んだのでバランスを崩してしまったのだろう。",
    );

    assert_debug_snapshot!("ep1_narrator2", snapshot);
}

#[test]
fn snapshot_ep1_kinzo3() {
    // tests correctness of squish/justify selection when max_width = layout_width
    let snapshot = make_snapshot(
        MessageTextLayout::Justify,
        MessageboxType::Ushiromiya,
        "右代宮\u{3000}金蔵@r@v01/11500136.「留弗夫の間抜けは女遊びばかりッ！！@k\u{3000}@v01/11500137.楼座はどこの馬の骨ともわからん男の赤ん坊など生みおって！！@k\u{3000}@v01/11500138.朱志香は無能で無学だ！！@k\u{3000}@v01/11500139.譲治には男としての器がない！@k\u{3000}@v01/11500140.戦人は右代宮家の栄誉を自ら捨ておった愚か者だッ！！@k\u{3000}@v01/11500141.真里亞など見るのも汚らわしいッ！！」",
    );

    assert_debug_snapshot!("ep1_kinzo3", snapshot);
}
