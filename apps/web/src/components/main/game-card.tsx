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
			<div className="flex flex-col-reverse sm:flex-row justify-between items-center gap-4 lg:gap-36 bg-card px-4 lg:px-17.5 py-10 lg:py-15.5 rounded-4xl w-full">
				<div>
					<h3 className="md:text-[40px] text-2xl truncate font-bold pb-7">
						{game.name}
					</h3>
					<p className="text-base truncate md:text-2xl sm:font-medium pb-4">
						{game.description}
					</p>
					<div className="flex flex-wrap gap-3 pb-4">
						{game.category && (
							<span className="text-xs md:text-sm bg-foreground/10 rounded-full py-2 md:py-2.5 px-4 md:px-5 md:font-medium">
								{game.category}
							</span>
						)}
					</div>
					<div className="text-sm md:text-xl md:font-medium flex flex-wrap gap-4">
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
					className="w-40 md:w-89.5"
				/>
			</div>
			{open && (
				<Button
					className="-translate-y-1/2 w-full max-w-28 md:max-w-80 rounded-full text-xs md:text-xl sm:font-medium"
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
