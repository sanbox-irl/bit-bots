// use super::{Component, ResourcesDatabase, SoundPlayer, SoundResource, SoundSource};
// use failure::Error;
// use rodio::{Decoder, Sink};
// use std::io::Cursor;

// pub fn play_sounds<'a>(
//     sound_sources: impl Iterator<Item = &'a mut Component<SoundSource>>,
//     sound_player: &mut SoundPlayer,
//     resources: &ResourcesDatabase,
// ) -> Result<(), Error> {
//     for sound_source_component in sound_sources {
//         let sound_source = sound_source_component.inner_mut();

//         if sound_source.muted == false {
//             if let Some(sound) = sound_source.sound_to_play {
//                 let sink: &Sink = match sound_player.sinks.iter().find(|&x| x.empty()) {
//                     Some(this_sink) => this_sink,
//                     None => {
//                         sound_player.sinks.push(Sink::new(&sound_player.device));
//                         info!(
//                             "Sound sink expanded -- we now have {} sinks.",
//                             sound_player.sinks.len()
//                         );
//                         &sound_player.sinks.last().unwrap()
//                     }
//                 };

//                 sink.append(get_sound(&sound, resources)?);
//                 sound_source.sound_to_play = None;
//             }
//         }
//     }

//     Ok(())
// }

// fn get_sound(
//     sound_resource: &SoundResource,
//     resources: &ResourcesDatabase,
// ) -> Result<Decoder<Cursor<&'static [u8]>>, Error> {
//     Ok(Decoder::new(
//         resources.sounds.get(&sound_resource).unwrap().clone(),
//     )?)
// }
