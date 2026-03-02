"use client";

import { useBackgroundMusic } from "@/lib/audio/use-background-music";

/**
 * Mounts background music management at the app root.
 * Renders no visible UI — just manages the audio lifecycle.
 */
export function AudioProvider({ children }: { children: React.ReactNode }) {
	useBackgroundMusic();
	return <>{children}</>;
}
