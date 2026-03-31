import Image from "next/image";
import Link from "next/link";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { ScrollArea, ScrollBar } from "@/components/ui/scroll-area";
import { ApiClient } from "@/lib/api/client";
import type { Game } from "@/lib/definitions";

export default async function GamesShowcase() {
	const { data: games } = await ApiClient.get<Game[]>("/api/games?order=asc");

	if (!games || games.length === 0) {
		return null;
	}

	return (
		<div className="mt-16 flex flex-col gap-6">
			<div className="flex flex-wrap justify-center gap-2 self-center md:gap-3">
				<Button className="rounded-full">Hot Games</Button>
				<Button className="bg-popover rounded-full">
					New Releases
				</Button>
				<Button className="bg-popover rounded-full">Top Rated</Button>
				<Button className="bg-popover rounded-full">Recommended</Button>
			</div>
			<ScrollArea className="w-full">
				<div className="flex gap-4">
					{games.map((game) => (
						<ShowcaseGameCard key={game.id} game={game} />
					))}
				</div>
				<ScrollBar orientation="horizontal" />
			</ScrollArea>
		</div>
	);
}

function ShowcaseGameCard({ game }: { game: Game }) {
	return (
		<Link
			href={`/games/${game.path}`}
			className="group w-70 shrink-0 overflow-hidden rounded-xl border border-white/10 bg-white/5 transition-all hover:border-white/20 hover:bg-white/[0.07]"
		>
			{/* Game Header */}
			<div className="flex items-center gap-3 border-b border-white/5 p-4">
				<Image
					src={game.imageUrl}
					alt={game.name}
					width={48}
					height={48}
					className="size-12 shrink-0 rounded-lg object-cover"
				/>
				<div className="min-w-0 flex-1">
					<h3 className="truncate text-sm font-bold sm:text-base">
						{game.name}
					</h3>
				</div>
			</div>

			{/* Content Section */}
			<div className="p-4">
				<p className="line-clamp-3 text-xs leading-relaxed text-gray-400">
					{game.description}
				</p>
			</div>
		</Link>
	);
}

export function GamesShowcaseSkeleton() {
	return (
		<div className="mt-16 flex flex-col gap-6">
			<div className="flex flex-wrap justify-center gap-2 self-center md:gap-3">
				{[1, 2, 3, 4].map((i) => (
					<Skeleton key={i} className="h-12 w-36 rounded-full" />
				))}
			</div>
			<ScrollArea className="w-full">
				<div className="flex gap-4 px-4">
					{[1, 2, 3].map((i) => (
						<div
							key={i}
							className="w-70 shrink-0 rounded-xl border border-white/10 bg-white/5"
						>
							{/* Header Skeleton */}
							<div className="flex items-center gap-3 border-b border-white/5 p-4">
								<Skeleton className="size-12 shrink-0 rounded-lg" />
								<div className="min-w-0 flex-1 space-y-2">
									<Skeleton className="h-4 w-24" />
									<Skeleton className="h-3 w-32" />
								</div>
							</div>
							{/* Content Skeleton */}
							<div className="space-y-2 p-4">
								<Skeleton className="h-3 w-full" />
								<Skeleton className="h-3 w-full" />
								<Skeleton className="h-3 w-3/4" />
							</div>
						</div>
					))}
				</div>
				<ScrollBar orientation="horizontal" />
			</ScrollArea>
		</div>
	);
}
