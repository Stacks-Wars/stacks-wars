"use client";

import { useEffect, useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { apiRequest } from "@/lib/api";
import { toast } from "sonner";
import { claimFtPoolReward, claimPoolReward } from "@/lib/actions/claimReward";
import { waitForTxConfirmed } from "@/lib/actions/waitForTxConfirmed";
import { Gift, Clock, Trophy, Loader2 } from "lucide-react";
import { PlayerLobbyInfo } from "@/types/schema/lobby";
import { formatNumber } from "@/lib/utils";

export default function UnclaimedRewards({ userId }: { userId: string }) {
	const [unclaimedLobbies, setUnclaimedLobbies] = useState<PlayerLobbyInfo[]>(
		[]
	);
	const [loading, setLoading] = useState(true);
	const [claimingLobby, setClaimingLobby] = useState<string | null>(null);

	useEffect(() => {
		const fetchUnclaimedRewards = async () => {
			try {
				const response = await apiRequest<PlayerLobbyInfo[]>({
					path: `/user/lobbies?user_id=${userId}&claim_state=not_claimed`,
					method: "GET",
				});
				setUnclaimedLobbies(response);
			} catch (error) {
				console.error("Failed to fetch unclaimed rewards:", error);
				toast.error("Failed to load unclaimed rewards");
			} finally {
				setLoading(false);
			}
		};

		fetchUnclaimedRewards();
	}, [userId]);

	const handleClaimReward = async (lobby: PlayerLobbyInfo) => {
		if (!lobby.contractAddress || !lobby.prizeAmount) {
			toast.error("Contract address is missing");
			return;
		}

		setClaimingLobby(lobby.id);
		try {
			const contract = lobby.contractAddress as `${string}.${string}`;
			let claimTxId: string | undefined;
			if (lobby.tokenSymbol !== "STX" && lobby.tokenId) {
				claimTxId = await claimFtPoolReward(
					contract,
					lobby.tokenId,
					lobby.prizeAmount
				);
			} else {
				claimTxId = await claimPoolReward(contract, lobby.prizeAmount);
			}

			if (!claimTxId) {
				throw new Error(
					"Failed to claim reward: missing transaction ID"
				);
			}

			await waitForTxConfirmed(claimTxId);

			await apiRequest({
				path: `/lobby/${lobby.id}/claim-state`,
				method: "PATCH",
				body: {
					claim: {
						status: "claimed",
						data: {
							tx_id: claimTxId,
						},
					},
				},
			});

			toast.success("Reward claimed successfully!");

			// Remove the claimed lobby from the list
			setUnclaimedLobbies((prev) =>
				prev.filter((l) => l.id !== lobby.id)
			);
		} catch (error) {
			console.error("Error claiming reward:", error);
			toast.error("Failed to claim reward. Please try again later.");
		} finally {
			setClaimingLobby(null);
		}
	};

	const formatDate = (dateString: string) => {
		return new Date(dateString).toLocaleDateString("en-US", {
			month: "short",
			day: "numeric",
			year: "numeric",
		});
	};

	const getRankIcon = (rank: number) => {
		switch (rank) {
			case 1:
				return "🥇";
			case 2:
				return "🥈";
			case 3:
				return "🥉";
			default:
				return `#${rank}`;
		}
	};

	if (loading) {
		return (
			<Card>
				<CardHeader>
					<CardTitle className="flex items-center gap-2">
						<Gift className="h-5 w-5 text-yellow-500" />
						Unclaimed Rewards
					</CardTitle>
				</CardHeader>
				<CardContent className="flex items-center justify-center py-8">
					<Loader2 className="h-6 w-6 animate-spin" />
				</CardContent>
			</Card>
		);
	}

	if (unclaimedLobbies.length === 0) {
		return (
			<Card>
				<CardHeader>
					<CardTitle className="flex items-center gap-2">
						<Gift className="h-5 w-5 text-yellow-500" />
						Unclaimed Rewards
					</CardTitle>
				</CardHeader>
				<CardContent className="text-center py-8">
					<div className="text-muted-foreground">
						<Trophy className="h-12 w-12 mx-auto mb-2 opacity-50" />
						<p>No unclaimed rewards</p>
						<p className="text-sm">
							All your rewards have been claimed!
						</p>
					</div>
				</CardContent>
			</Card>
		);
	}

	return (
		<Card>
			<CardHeader>
				<CardTitle className="flex items-center gap-2">
					<Gift className="h-5 w-5 text-yellow-500" />
					Unclaimed Rewards
					<Badge variant="secondary" className="ml-auto">
						{unclaimedLobbies.length}
					</Badge>
				</CardTitle>
			</CardHeader>
			<CardContent className="space-y-4">
				{unclaimedLobbies.map(
					(lobby) =>
						lobby.prizeAmount !== 0 &&
						lobby.contractAddress &&
						lobby.rank && (
							<div
								key={lobby.id}
								className="flex items-center justify-between p-4 border rounded-lg bg-muted/50"
							>
								<div className="flex-1 min-w-0">
									<div className="flex items-center gap-2 mb-1">
										<h4 className="font-medium truncate">
											{lobby.name}
										</h4>
										<Badge
											variant="outline"
											className="shrink-0"
										>
											{getRankIcon(lobby.rank)}
										</Badge>
									</div>
									<div className="flex items-center gap-4 text-sm text-muted-foreground">
										<div className="flex items-center gap-1">
											<Clock className="h-3 w-3" />
											{formatDate(lobby.createdAt)}
										</div>
										<div className="text-green-600 font-medium">
											+
											{formatNumber(
												lobby.prizeAmount || 0
											)}{" "}
											{lobby.tokenSymbol}
										</div>
									</div>
								</div>
								<Button
									onClick={() => handleClaimReward(lobby)}
									disabled={claimingLobby === lobby.id}
									className="shrink-0"
								>
									{claimingLobby === lobby.id ? (
										<>
											<Loader2 className="h-4 w-4 animate-spin mr-2" />
											Claiming...
										</>
									) : (
										<>
											<Gift className="h-4 w-4 mr-2" />
											Claim
										</>
									)}
								</Button>
							</div>
						)
				)}
			</CardContent>
		</Card>
	);
}
