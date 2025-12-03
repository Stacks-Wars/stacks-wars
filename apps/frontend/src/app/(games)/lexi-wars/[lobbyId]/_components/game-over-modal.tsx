import {
	Dialog,
	DialogContent,
	DialogHeader,
	DialogTitle,
	DialogDescription,
	DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { useEffect, useState } from "react";
import { truncateAddress } from "@/lib/utils";
import { useRouter } from "next/navigation";
import { ArrowLeft, Trophy } from "lucide-react";
import { PlayerStanding } from "@/types/schema/player";

interface GameOverModalProps {
	standing: PlayerStanding[] | null;
	userId: string;
	contractAddress: string | null;
	isClaimed: boolean;
	creatorId: string;
}

export default function GameOverModal({
	standing,
	userId,
	contractAddress,
	isClaimed,
	creatorId,
}: GameOverModalProps) {
	const [open, setOpen] = useState(false);
	const [countdown, setCountdown] = useState(30);
	const router = useRouter();

	useEffect(() => {
		const shouldOpen = standing && (contractAddress ? isClaimed : true);

		if (shouldOpen) {
			setOpen(true);
			const timer = setInterval(() => {
				setCountdown((prev) => {
					if (prev <= 1) {
						clearInterval(timer);
						router.replace("/lobby");
						return 0;
					}
					return prev - 1;
				});
			}, 1000);

			return () => clearInterval(timer);
		}
	}, [standing, router, contractAddress, isClaimed]);

	return (
		<Dialog open={open} onOpenChange={setOpen}>
			<DialogContent
				className="from-primary/90 to-primary/70 border-0 bg-gradient-to-b text-white sm:max-w-md"
				hideClose
				disableOutsideClose
			>
				<DialogHeader className="text-center">
					<div className="mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-white/20 p-2">
						<Trophy className="h-8 w-8 text-yellow-300" />
					</div>
					<DialogTitle className="text-2xl font-bold">
						Game Over!
					</DialogTitle>
					<DialogDescription className="text-white/80">
						Tournament results are in
					</DialogDescription>
				</DialogHeader>

				<div className="mt-6 space-y-3">
					{standing?.map((s) => {
						const isCurrentUser = s.player.id === userId;
						const isCreator = s.player.id === creatorId;

						return (
							<div
								key={s.player.id}
								className={`flex items-center justify-between rounded-lg px-4 py-3 transition-all ${
									isCurrentUser
										? "bg-white/20 shadow-lg"
										: "bg-white/10"
								}`}
							>
								<div className="flex min-w-0 flex-1 items-center space-x-3">
									<span className="flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-white/20 text-sm font-medium">
										{s.rank}
									</span>
									<div className="flex min-w-0 flex-1 items-center gap-2">
										<span className="truncate font-medium">
											{s.player.user.displayName ||
												s.player.user.username ||
												truncateAddress(
													s.player.user.walletAddress
												)}
										</span>
										<div className="flex shrink-0 gap-1">
											{isCurrentUser && (
												<Badge
													variant="secondary"
													className="bg-white/30 px-2 py-0.5 text-xs text-white hover:bg-white/40"
												>
													You
												</Badge>
											)}
											{isCreator && (
												<Badge
													variant="default"
													className="bg-yellow-500/80 px-2 py-0.5 text-xs text-white hover:bg-yellow-600/80"
												>
													Creator
												</Badge>
											)}
										</div>
									</div>
								</div>
								<div className="flex shrink-0 items-center space-x-2">
									<span className="text-sm text-white/70">
										Rank
									</span>
									<span className="font-bold">
										{s.player.rank}
									</span>
								</div>
							</div>
						);
					})}
				</div>

				<DialogFooter className="mt-8">
					<Button
						onClick={() => router.push("/lobby")}
						className="text-primary hover:text-primary/90 w-full bg-white hover:bg-white/90"
						size="lg"
					>
						<div className="flex items-center justify-center space-x-2">
							<ArrowLeft className="h-4 w-4" />
							<span>Back to Lobby ({countdown}s)</span>
						</div>
					</Button>
				</DialogFooter>
			</DialogContent>
		</Dialog>
	);
}
