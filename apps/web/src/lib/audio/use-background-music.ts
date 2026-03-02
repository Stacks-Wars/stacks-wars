"use client";

import { useEffect, useRef, useCallback } from "react";
import { useMusicEnabled, useMusicVolume } from "@/lib/stores/app";

const BG_TRACKS = [
	"/audio/bg/Alexander Ehlers - Doomed.mp3",
	"/audio/bg/Alexander Ehlers - Flags.mp3",
	"/audio/bg/Alexander Ehlers - Great mission.mp3",
	"/audio/bg/Alexander Ehlers - Spacetime.mp3",
	"/audio/bg/Alexander Ehlers - Twists.mp3",
	"/audio/bg/Alexander Ehlers - Waking the devil.mp3",
	"/audio/bg/Alexander Ehlers - Warped.mp3",
];

function shuffle<T>(arr: T[]): T[] {
	const shuffled = [...arr];
	for (let i = shuffled.length - 1; i > 0; i--) {
		const j = Math.floor(Math.random() * (i + 1));
		[shuffled[i], shuffled[j]] = [shuffled[j], shuffled[i]];
	}
	return shuffled;
}

/**
 * Hook that manages background music playback.
 * Loops through shuffled bg tracks, respecting user's music preferences.
 * Handles browser autoplay restrictions by resuming on first user interaction.
 */
export function useBackgroundMusic() {
	const musicEnabled = useMusicEnabled();
	const musicVolume = useMusicVolume();
	const audioRef = useRef<HTMLAudioElement | null>(null);
	const tracksRef = useRef<string[]>(shuffle(BG_TRACKS));
	const indexRef = useRef(0);
	const hasInteractedRef = useRef(false);

	const playNext = useCallback(() => {
		indexRef.current = (indexRef.current + 1) % tracksRef.current.length;
		if (indexRef.current === 0) {
			tracksRef.current = shuffle(BG_TRACKS);
		}
		const audio = audioRef.current;
		if (audio) {
			audio.src = tracksRef.current[indexRef.current];
			audio.play().catch(() => {});
		}
	}, []);

	// Create audio element once
	useEffect(() => {
		const audio = new Audio();
		audio.addEventListener("ended", playNext);
		audioRef.current = audio;

		return () => {
			audio.pause();
			audio.removeEventListener("ended", playNext);
			audio.src = "";
			audioRef.current = null;
		};
	}, [playNext]);

	// Handle musicEnabled toggle
	useEffect(() => {
		const audio = audioRef.current;
		if (!audio) return;

		if (musicEnabled) {
			if (!audio.src || audio.src === window.location.href) {
				audio.src = tracksRef.current[indexRef.current];
			}
			audio.play().catch(() => {});
		} else {
			audio.pause();
		}
	}, [musicEnabled]);

	// Handle volume changes
	useEffect(() => {
		if (audioRef.current) {
			audioRef.current.volume = musicVolume;
		}
	}, [musicVolume]);

	// Resume playback after first user interaction (browser autoplay policy)
	useEffect(() => {
		if (!musicEnabled) return;

		const handleInteraction = () => {
			if (hasInteractedRef.current) return;
			hasInteractedRef.current = true;

			const audio = audioRef.current;
			if (audio && audio.paused && musicEnabled) {
				if (!audio.src || audio.src === window.location.href) {
					audio.src = tracksRef.current[indexRef.current];
				}
				audio.play().catch(() => {});
			}

			document.removeEventListener("click", handleInteraction);
			document.removeEventListener("keydown", handleInteraction);
		};

		document.addEventListener("click", handleInteraction);
		document.addEventListener("keydown", handleInteraction);

		return () => {
			document.removeEventListener("click", handleInteraction);
			document.removeEventListener("keydown", handleInteraction);
		};
	}, [musicEnabled]);
}
