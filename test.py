from evalpy import TrackObject, MidiObject

midi = MidiObject("./data/maestro300/2004/MIDI-Unprocessed_SMF_02_R1_2004_01-05_ORIG_MID--AUDIO_02_R1_2004_05_Track05_wav.midi")
track = midi[1]

features = track.feature_space()
features.draw_ssm("test_ssm.png", .01)