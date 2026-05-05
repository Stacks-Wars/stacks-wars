import { getAudioState } from "@/lib/stores/app";

/**
 * Play a one-shot sound effect.
 * Respects the user's SFX enabled/volume preferences from the app store.
 */
export function playSound(src: string = "/audio/click.wav") {
	const { sfxEnabled, sfxVolume } = getAudioState();
	if (!sfxEnabled) return;

	const audio = new Audio(src);
	audio.volume = sfxVolume;
	audio.play().catch(() => {});
}
