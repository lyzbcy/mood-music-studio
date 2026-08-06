// 防未使用告警：main 仅转发到 lib::run
#![allow(dead_code)]

fn main() {
    mood_music_studio_lib::run()
}
