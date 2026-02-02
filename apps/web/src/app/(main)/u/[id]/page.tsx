import NotFound from "@/app/not-found";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { ApiClient } from "@/lib/api/client";
import type { User, Game, LeaderBoard, LobbyInfo } from "@/lib/definitions";
import { formatAddress } from "@/lib/utils";
import Image from "next/image";
import EditProfile from "./_components/edit-profile";
import dynamic from "next/dynamic";
import PlayerStats from "./_components/player-stats";
import UnclaimedRewards from "./_components/unclaimed-rewards";
import PlayerLobbies from "./_components/player-lobbies";
import CreatedGames from "./_components/created-games";

const LogoutButton = dynamic(() => import("./_components/logout-button"));

export default async function UserProfile({
	params,
}: {
	params: Promise<{ id: string }>;
}) {
	const id = (await params).id;

	const response = await ApiClient.get<User>(`/api/user/${id}`);

	if (!response.data) {
		return <NotFound />;
	}
	const user = response.data;

	// Run remaining requests in parallel
	const [gamesResult, statsResult, lobbiesResult] = await Promise.allSettled([
		ApiClient.get<Game[]>(`/api/game/by-creator/${user.id}`),
		ApiClient.get<LeaderBoard>(`/api/leaderboard/${user.id}`),
		ApiClient.get<{ 0: LobbyInfo[]; 1: number }>(
			`/api/player-lobby/${user.id}?status=waiting,starting,inProgress&limit=6&offset=0`
		),
	]);

	const games =
		gamesResult.status === "fulfilled" ? gamesResult.value.data || [] : [];
	const playerStats =
		statsResult.status === "fulfilled" ? statsResult.value.data : null;
	const initialLobbies =
		lobbiesResult.status === "fulfilled"
			? lobbiesResult.value.data?.[0] || []
			: [];
	const initialLobbiesTotal =
		lobbiesResult.status === "fulfilled"
			? lobbiesResult.value.data?.[1] || 0
			: 0;

	return (
		<div className="container mx-auto sm:px-4">
			<div className="flex flex-col">
				<Image
					src={"/images/cover.svg"}
					alt="cover photo"
					width={1240}
					height={280}
					className="h-35 w-full object-cover sm:h-70 sm:rounded-4xl"
				/>
				<div className="flex justify-between px-4">
					<Avatar className="border-background -mb-12.5 size-25 translate-x-10 -translate-y-1/2 rounded-full text-3xl sm:-mb-22.5 sm:size-45 sm:translate-x-20 sm:border-4 sm:text-6xl">
						<AvatarImage
							src={user.profileImage}
							alt="profile photo"
							width={180}
							height={180}
						/>
						<AvatarFallback>
							{(
								user.displayName ||
								user.username ||
								user.walletAddress
							)
								.slice(0, 2)
								.toUpperCase()}
						</AvatarFallback>
					</Avatar>
					<div className="flex gap-2">
						<EditProfile userProfile={user} />

						<LogoutButton userProfile={user} />
					</div>
				</div>
			</div>
			<div className="mt-4 w-full max-w-full space-y-1 px-4 text-center sm:mt-7 sm:text-left">
				<p className="w-full truncate text-xl font-bold sm:text-4xl">
					{user.displayName}
				</p>
				{user.username ? (
					<p className="w-full truncate text-sm font-medium sm:text-2xl">
						@{user.username}{" "}
						<span className="text-foreground/70 text-xs font-normal sm:text-xl">
							({formatAddress(user.walletAddress)})
						</span>
					</p>
				) : (
					<p className="w-full truncate">{user.walletAddress}</p>
				)}
			</div>
			{playerStats && (
				<div className="mt-8 px-4 sm:mt-12 sm:px-0">
					<PlayerStats stats={playerStats} />
				</div>
			)}
			<UnclaimedRewards userId={user.id} />
			<PlayerLobbies
				userId={user.id}
				initialLobbies={initialLobbies}
				initialTotal={initialLobbiesTotal}
			/>
			<CreatedGames userId={user.id} games={games} />
		</div>
	);
}
