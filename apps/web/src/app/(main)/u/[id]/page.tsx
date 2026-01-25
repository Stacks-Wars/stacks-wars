import NotFound from "@/app/not-found";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { Button } from "@/components/ui/button";
import { ApiClient } from "@/lib/api/client";
import type { User, Game } from "@/lib/definitions";
import { formatAddress } from "@/lib/utils";
import Image from "next/image";
import { FiEdit3 } from "react-icons/fi";
import { getAuthenticatedUserId } from "@/lib/auth/jwt";
import EditProfile from "./_components/edit-profile";
import LogoutButton from "./_components/logout-button";
import GameCard from "@/components/main/game-card";
import Link from "next/link";
import { IoAdd } from "react-icons/io5";

export default async function page({
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

	// Check if this is the current user's profile
	const currentUserId = await getAuthenticatedUserId();
	const isOwnProfile = currentUserId === user.id;

	// Fetch user's created games
	const gamesResponse = await ApiClient.get<Game[]>(
		`/api/game/by-creator/${user.id}`
	);
	const games = gamesResponse.data || [];

	return (
		<div className="container mx-auto sm:px-4">
			<div className="flex flex-col">
				<Image
					src={"/images/cover.svg"}
					alt="cover photo"
					width={1240}
					height={280}
					className="h-35 sm:h-70 sm:rounded-4xl w-full object-cover"
				/>
				<div className="flex justify-between px-4">
					<Avatar className="rounded-full -translate-y-1/2 translate-x-10 sm:translate-x-20 -mb-12.5 sm:-mb-22.5 sm:border-4 border-background size-25 sm:size-45 text-3xl sm:text-6xl">
						<AvatarImage
							//src={"/images/avatar.svg"}
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
					{isOwnProfile && (
						<div className="flex gap-2">
							<EditProfile currentUser={user}>
								<Button className="rounded-full text-xs sm:text-base bg-muted hover:bg-muted/90 h-6 sm:h-12 has-[>svg]:px-3.5 sm:has-[>svg]:px-7 -translate-y-1/2">
									<FiEdit3 /> Edit Profile
								</Button>
							</EditProfile>
							<LogoutButton />
						</div>
					)}
				</div>
			</div>
			<div className="mt-4 sm:mt-7 space-y-1 w-full max-w-full px-4 text-center sm:text-left">
				<p className="text-xl sm:text-4xl font-bold truncate w-full">
					{user.displayName}
				</p>
				{user.username ? (
					<p className="text-sm sm:text-2xl font-medium truncate w-full ">
						@{user.username}{" "}
						<span className="font-normal text-xs sm:text-xl text-foreground/70">
							({formatAddress(user.walletAddress)})
						</span>
					</p>
				) : (
					<p className="truncate w-full">{user.walletAddress}</p>
				)}
			</div>
			{/* Player Rank */}
			{/* Player Active Lobbies */}
			{/* Private user uncliamed rewards */}
			{(games.length > 0 || isOwnProfile) && (
				<div className="mt-8 sm:mt-12 px-4 sm:px-0">
					<div className="flex justify-between items-center mb-4 sm:mb-6">
						<h2 className="text-xl sm:text-3xl font-bold">
							{isOwnProfile ? "My Games" : "Created Games"}
						</h2>
						{isOwnProfile && (
							<Button
								asChild
								className="rounded-full text-xs sm:text-base h-8 sm:h-12 has-[>svg]:px-3.5 sm:has-[>svg]:px-7"
							>
								<Link href="/create-game">
									<IoAdd className="text-lg sm:text-xl" />{" "}
									Create Game
								</Link>
							</Button>
						)}
					</div>
					{games.length > 0 ? (
						<div className="grid grid-cols-1 gap-4 sm:gap-6">
							{games.map((game) => (
								<GameCard key={game.id} game={game} />
							))}
						</div>
					) : (
						<div className="text-center py-8 sm:py-12 text-muted-foreground">
							<p className="text-sm sm:text-base">
								No games created yet
							</p>
						</div>
					)}
				</div>
			)}
		</div>
	);
}
