"use client";

import { useState } from "react";
import { Music, Volume2, VolumeX, Zap } from "lucide-react";
import { Switch } from "@/components/ui/switch";
import { Slider } from "@/components/ui/slider";
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogHeader,
	DialogTitle,
	DialogTrigger,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import {
	useMusicEnabled,
	useMusicVolume,
	useSfxEnabled,
	useSfxVolume,
	useAppActions,
} from "@/lib/stores/app";
import { useUser } from "@/lib/stores/user";
import type { User } from "@/lib/definitions";

interface AudioSettingsProps {
	userProfile: User;
}

export default function AudioSettings({ userProfile }: AudioSettingsProps) {
	const [open, setOpen] = useState(false);
	const user = useUser();
	const musicEnabled = useMusicEnabled();
	const musicVolume = useMusicVolume();
	const sfxEnabled = useSfxEnabled();
	const sfxVolume = useSfxVolume();
	const { setMusicEnabled, setMusicVolume, setSfxEnabled, setSfxVolume } =
		useAppActions();

	if (!user || user.id !== userProfile.id) return null;

	return (
		<Dialog open={open} onOpenChange={setOpen}>
			<DialogTrigger asChild>
				<Button className="bg-muted hover:bg-muted/90 h-6 -translate-y-1/2 rounded-full text-xs has-[>svg]:px-3.5 sm:h-12 sm:text-base sm:has-[>svg]:px-7">
					<Volume2 /> <span className="hidden sm:inline">Audio</span>
				</Button>
			</DialogTrigger>
			<DialogContent className="rounded-4xl sm:max-w-106.25">
				<DialogHeader>
					<DialogTitle className="text-xl sm:text-2xl">
						Audio Settings
					</DialogTitle>
					<DialogDescription className="text-sm sm:text-base">
						Customize music and sound effects.
					</DialogDescription>
				</DialogHeader>

				<div className="space-y-6">
					{/* Background Music */}
					<div className="space-y-3">
						<div className="flex items-center justify-between">
							<div className="flex items-center gap-3">
								<Music className="size-5 text-purple-500 sm:size-6" />
								<div>
									<p className="text-sm font-medium sm:text-base">
										Background Music
									</p>
									<p className="text-xs text-gray-400 sm:text-sm">
										Loop through background tracks
									</p>
								</div>
							</div>
							<Switch
								checked={musicEnabled}
								onCheckedChange={setMusicEnabled}
							/>
						</div>
						{musicEnabled && (
							<div className="flex items-center gap-3 pl-8">
								{musicVolume === 0 ? (
									<VolumeX className="size-4 shrink-0 text-gray-400" />
								) : (
									<Volume2 className="size-4 shrink-0 text-gray-400" />
								)}
								<Slider
									value={[Math.round(musicVolume * 100)]}
									onValueChange={(v) =>
										setMusicVolume(v[0] / 100)
									}
									max={100}
									step={1}
									className="w-full"
								/>
								<span className="w-8 shrink-0 text-right text-xs text-gray-400">
									{Math.round(musicVolume * 100)}%
								</span>
							</div>
						)}
					</div>

					{/* Sound Effects */}
					<div className="space-y-3">
						<div className="flex items-center justify-between">
							<div className="flex items-center gap-3">
								<Zap className="size-5 text-blue-500 sm:size-6" />
								<div>
									<p className="text-sm font-medium sm:text-base">
										Sound Effects
									</p>
									<p className="text-xs text-gray-400 sm:text-sm">
										Button clicks and game sounds
									</p>
								</div>
							</div>
							<Switch
								checked={sfxEnabled}
								onCheckedChange={setSfxEnabled}
							/>
						</div>
						{sfxEnabled && (
							<div className="flex items-center gap-3 pl-8">
								{sfxVolume === 0 ? (
									<VolumeX className="size-4 shrink-0 text-gray-400" />
								) : (
									<Volume2 className="size-4 shrink-0 text-gray-400" />
								)}
								<Slider
									value={[Math.round(sfxVolume * 100)]}
									onValueChange={(v) =>
										setSfxVolume(v[0] / 100)
									}
									max={100}
									step={1}
									className="w-full"
								/>
								<span className="w-8 shrink-0 text-right text-xs text-gray-400">
									{Math.round(sfxVolume * 100)}%
								</span>
							</div>
						)}
					</div>
				</div>
			</DialogContent>
		</Dialog>
	);
}
