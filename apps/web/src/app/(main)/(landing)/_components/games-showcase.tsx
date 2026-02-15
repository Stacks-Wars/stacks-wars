import Image from "next/image";
import Link from "next/link";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { ApiClient } from "@/lib/api/client";
import type { Game } from "@/lib/definitions";

export default async function GamesShowcase() {
	const { data: games } = await ApiClient.get<Game[]>("/api/games?order=asc");
	console.log(games);

	if (!games || games.length === 0) {
		return null;
	}

	return (
		<div className="mt-16 flex flex-col gap-6">
			<div className="flex flex-wrap justify-center gap-2 self-center md:gap-3">
				<Button className="rounded-full">Hot Games</Button>
				<Button className="bg-popover rounded-full">New Releases</Button>
				<Button className="bg-popover rounded-full">Top Rated</Button>
				<Button className="bg-popover rounded-full">Recommended</Button>
			</div>
			<div className="relative -mx-4">
				<div className="scrollbar-hide overflow-x-auto px-4 pb-4">
					<div className="flex gap-4">
						{games.map((game) => (
							<ShowcaseGameCard key={game.id} game={game} />
						))}
					</div>
				</div>
			</div>
			<Image
				src={"/images/footer-seperator.svg"}
				alt="Footer Illustration"
				width={1248}
				height={28}
				className="mb-6 h-4 w-full object-cover sm:mb-8 sm:h-7 lg:mb-12"
			/>
		</div>
	);
}

function ShowcaseGameCard({ game }: { game: Game }) {
	return (
		<Link
			href={`/game/${game.path}`}
			className="group border-border/50 relative h-[186px] w-[240px] shrink-0 overflow-hidden rounded-2xl border md:h-[334px] md:w-[445px]"
		>
			<Image
				src={game.imageUrl}
				alt={game.name}
				fill
				className="object-cover transition-transform duration-300 group-hover:scale-105"
			/>
			<div className="absolute inset-0 flex flex-col justify-end bg-gradient-to-t from-black/80 to-transparent p-4 opacity-0 transition-opacity duration-300 group-hover:opacity-100 md:p-6">
				<h3 className="text-lg font-bold text-white md:text-2xl">
					{game.name}
				</h3>
				<p className="line-clamp-2 text-xs text-white/80 md:text-sm">
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
					<Skeleton key={i} className="h-10 w-28 rounded-full" />
				))}
			</div>
			<div className="relative -mx-4">
				<div className="scrollbar-hide overflow-x-auto px-4 pb-4">
					<div className="flex gap-4">
						{[1, 2, 3, 4, 5].map((i) => (
							<Skeleton
								key={i}
								className="h-[186px] w-[240px] shrink-0 rounded-2xl md:h-[334px] md:w-[445px]"
							/>
						))}
					</div>
				</div>
			</div>
			<Skeleton className="mb-6 h-4 w-full sm:mb-8 sm:h-7 lg:mb-12" />
		</div>
	);
}
