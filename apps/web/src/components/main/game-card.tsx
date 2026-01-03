import Image from "next/image";
import { Button } from "../ui/button";
import Link from "next/link";
import type { Game } from "@/lib/definitions";
import { IoStar } from "react-icons/io5";

export default function GameCard({
	game,
	open,
}: {
	game: Game;
	open?: "gamePage" | "createLobbyPage";
}) {
	return (
		<div className="flex flex-col items-center w-full">
			<div className="flex flex-col-reverse sm:flex-row border justify-between sm:items-center bg-gradient-primary px-4 lg:px-17.5 py-10 lg:py-15.5 rounded-4xl w-full">
				<div className="sm:w-1/2 space-y-4 sm:space-y-2 md:space-y-4">
					<h3 className="lg:text-[40px] text-2xl w-full truncate font-bold pb-3">
						{game.name}
					</h3>
					<p className="text-base line-clamp-2 lg:text-2xl sm:font-medium">
						{game.description}
					</p>
					<div className="flex gap-3 w-full overflow-hidden">
						{game.category && (
							<span className="text-xs lg:text-sm bg-foreground/10 rounded-full py-2 md:py-2.5 px-4 md:px-5 md:font-medium">
								{game.category}
							</span>
						)}
					</div>
					<div className="text-sm lg:text-xl lg:font-medium flex gap-4 w-full truncate">
						<p>
							<span className="font-medium ">Active Rooms:</span>{" "}
							<span>3</span>
						</p>
						<p>
							<span className="font-medium">Ratings:</span>{" "}
							<span>4.5</span>
						</p>
						<p className="flex items-center gap-1">
							<span className="font-medium">Volume:</span>{" "}
							<span>1K STX</span>{" "}
							<IoStar className="text-yellow-400" />
						</p>
					</div>
				</div>
				<Image
					src={game.imageUrl}
					alt="game logo"
					width={358}
					height={182}
					loading="lazy"
					className="max-w-40 md:max-w-89.5 w-full self-center"
				/>
			</div>
			{open && (
				<Button
					className="-translate-y-1/2 w-full max-w-28 lg:max-w-80 bg-muted-gradient rounded-full text-xs lg:text-xl sm:font-medium -mb-6 lg:-mb-7.5 transition hover:opacity-90"
					asChild
				>
					{open === "createLobbyPage" ? (
						<Link href={`/games/${game.path}`}>Play Now</Link>
					) : (
						<Link href={{ pathname: `/game/${game.path}` }}>
							View Game
						</Link>
					)}
				</Button>
			)}
		</div>
	);
}
