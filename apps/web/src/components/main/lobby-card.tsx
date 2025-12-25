import type { Lobby } from "@/lib/definitions";
import { Button } from "../ui/button";
import Image from "next/image";
import { Badge } from "../ui/badge";
import { Lock, LockOpen, Users } from "lucide-react";
import { BiCoinStack } from "react-icons/bi";

export default function LobbyCard({ lobby }: { lobby: Lobby }) {
	return (
		<div className="bg-card p-6 lg:p-12 rounded-4xl border max-w-152.5 space-y-4 lg:space-y-8">
			<div className="space-y-4 lg:space-y-6.5">
				<div className="flex justify-between items-center">
					<div>
						<p className="truncate text-xl lg:text-2xl font-medium">
							{lobby.name}
						</p>
						<p className="truncate text-xs lg:text-lg text-foreground/50">
							Creator -{" "}
							<span className="text-foreground">
								@{lobby.creatorId}
							</span>
						</p>
					</div>
					<Badge className="py-2.5 lg:py-3.5 px-3.5 lg:px-6.5 text-xs lg:text-sm font-medium">
						{lobby.status}
					</Badge>
				</div>
				<div className="text-xs lg:text-base flex items-center justify-center gap-3 lg:gap-5">
					{lobby.isPrivate ? (
						<p className="flex items-center gap-1 lg:gap-2.5">
							<Lock size={20} className="size-4 lg:size-5" />{" "}
							<span>Private</span>
						</p>
					) : (
						<p className="flex items-center gap-1 lg:gap-2.5">
							<LockOpen size={20} className="size-4 lg:size-5" />
							<span>Public</span>
						</p>
					)}
					<p className="flex items-center gap-1 lg:gap-2.5">
						<Users size={20} className="size-4 lg:size-5" />
						<span>4</span>
					</p>
					{lobby.currentAmount && (
						<p className="flex items-center gap-1 lg:gap-2.5">
							<BiCoinStack
								size={20}
								className="size-4 lg:size-5"
							/>
							<span>
								{lobby.currentAmount}
								{lobby.tokenSymbol}
							</span>
						</p>
					)}
					{lobby.entryAmount && (
						<p>
							Entry Fee: {lobby.entryAmount}
							{lobby.tokenSymbol}
						</p>
					)}
				</div>
				<Image
					src={"/images/lexi-wars.svg"}
					alt="game-cover"
					width={516}
					height={185}
					loading="lazy"
					className="w-full h-30 lg:h-45 rounded-3xl"
				/>
				<p className="text-sm lg:text-xl">{lobby.description}</p>
			</div>
			<Button
				variant={"secondary"}
				className="rounded-full w-full text-base lg:text-xl font-medium py-2.5 lg:py-6"
			>
				Open Room
			</Button>
		</div>
	);
}
