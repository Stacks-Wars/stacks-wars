import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import type { LeaderBoard } from "@/lib/definitions";
import { displayUserIdentifier } from "@/lib/utils";

interface PodiumPlayer {
	rank: number;
	name: string;
	wins: number;
	pts: number;
	image?: string;
}

export default function LeaderBoardPodium({
	leaderboard,
}: {
	leaderboard: LeaderBoard[];
}) {
	if (!leaderboard || leaderboard.length === 0) return null;

	// Map leaderboard to podium format
	const podiumPlayers: PodiumPlayer[] = leaderboard
		.slice(0, 3)
		.map((p, i) => ({
			rank: i + 1,
			name: displayUserIdentifier(p),
			wins: p.totalWins ?? p.totalMatches ?? 0,
			pts: p.points ?? 0,
			image: p.profileImage,
		}));

	const players = podiumPlayers;
	const first = players.find((p) => p.rank === 1);
	const second = players.find((p) => p.rank === 2);
	const third = players.find((p) => p.rank === 3);

	return (
		<div className="mx-auto flex min-h-100 w-full max-w-5xl items-end justify-center gap-2 bg-black md:min-h-150 md:gap-6">
			{/* 2nd Place */}
			{second && (
				<PodiumCard player={second} height="h-[280px] md:h-[400px]" />
			)}

			{/* 1st Place (Center) */}
			{first && (
				<PodiumCard
					player={first}
					height="h-[350px] md:h-[500px]"
					isFirst
				/>
			)}

			{/* 3rd Place */}
			{third && (
				<PodiumCard player={third} height="h-[280px] md:h-[400px]" />
			)}
		</div>
	);
}

const rankStyles: Record<number, string> = {
	1: "from-yellow-300 via-yellow-500 to-yellow-600 border-yellow-200/50 shadow-yellow-500/20",
	2: "from-slate-200 via-gray-400 to-gray-500 border-gray-100/50 shadow-gray-400/20",
	3: "from-orange-300 via-orange-500 to-orange-700 border-orange-200/50 shadow-orange-500/20",
};

const defaultStyle = "from-gray-300 to-gray-500 border-white/20 shadow-lg";

function PodiumCard({
	player,
	height,
	isFirst,
}: {
	player: PodiumPlayer;
	height: string;
	isFirst?: boolean;
}) {
	const currentStyle = rankStyles[player.rank] || defaultStyle;
	return (
		<div
			className={`relative flex max-w-60 min-w-25 flex-1 flex-col items-center rounded-t-full border-x border-t border-white/10 bg-linear-to-b from-[#1E2245]/40 to-transparent transition-all duration-300 ${height}`}
		>
			<div className="absolute -top-4 scale-75 md:-top-6 md:scale-100">
				<div className="relative flex items-center justify-center">
					<div
						className={`flex size-12 rotate-45 items-center justify-center rounded-lg border-2 bg-linear-to-br shadow-lg transition-all md:h-16 md:w-16 ${currentStyle}`}
					>
						<span className="-rotate-45 text-base font-bold text-white md:text-xl">
							{player.rank}
						</span>
					</div>
				</div>
			</div>

			{/* Avatar Section */}
			<div className="relative mt-8 md:mt-12">
				<div
					className={`rounded-full border-2 p-0.5 md:p-1 ${isFirst ? "border-blue-500" : "border-blue-900/50"}`}
				>
					<Avatar
						className={`${isFirst ? "h-20 w-20 md:h-32 md:w-32" : "h-16 w-16 md:h-28 md:w-28"} border-2 border-black md:border-4`}
					>
						<AvatarImage src={player.image} />
						<AvatarFallback className="bg-[#0A0A0A] text-xs text-white md:text-base">
							{player.name.slice(0, 2)}
						</AvatarFallback>
					</Avatar>
				</div>

				{/* Mobile Rank Bubble */}
				<div className="absolute right-0 bottom-1 flex h-5 w-5 items-center justify-center rounded-full border border-black bg-[#3B4CC0] text-[10px] font-bold text-white md:h-8 md:w-8 md:text-sm">
					{player.rank}
				</div>
			</div>

			{/* Player Stats */}
			<div className="mt-4 px-1 text-center md:mt-6">
				<p className="text-[8px] tracking-widest text-gray-500 uppercase md:text-xs">
					Player Name
				</p>
				<p className="max-w-20 truncate text-xs font-bold text-white md:max-w-full md:text-lg">
					{player.name}
				</p>

				<div className="mt-2 flex flex-col items-center gap-2 md:mt-4 md:flex-row md:gap-8">
					<div className="text-center">
						<p className="text-[7px] text-gray-500 uppercase md:text-[10px]">
							Game Wins
						</p>
						<p className="text-[10px] font-bold text-white md:text-sm">
							{player.wins}
						</p>
					</div>
					<div className="text-center">
						<p className="text-[7px] text-gray-500 uppercase md:text-[10px]">
							Wars Points
						</p>
						<p className="text-[10px] font-bold text-white md:text-sm">
							{player.pts} WARS
						</p>
					</div>
				</div>
			</div>
		</div>
	);
}
