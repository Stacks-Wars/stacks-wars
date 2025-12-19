import Image from "next/image";
import { Button } from "../ui/button";
import Link from "next/link";
import type { Game } from "@/lib/definitions";

export default function GameCard(game: Game) {
	return (
		<div className="flex flex-col items-center">
			<div className="flex justify-between items-center lg:gap-36 gap-10 bg-card lg:p-20 p-5 lg:rounded-xl rounded-[12px]">
				<div>
					<h3 className="lg:text-[40px]/10 text-[10px]/[10px] font-medium lg:mb-4 mb-1">
						{game.name}
					</h3>
					<p className="lg:text-2xl/6 text-[6.5px]/[6.5px] lg:mb-7 mb-2">
						{game.description}
					</p>
					<div className="flex lg:gap-4 gap-1 lg:mb-9 mb-2">
						{game.category && (
							<span className="lg:text-xl/6 text-[6px] bg-foreground/10 rounded-full lg:py-4 py-1 lg:px-7 px-2">
								{game.category}
							</span>
						)}
					</div>
					<div className="lg:text-xl text-[5px] flex gap-5">
						<p>
							<span className="font-medium ">Active Rooms:</span>{" "}
							<span>3</span>
						</p>
						<p>
							<span className="f ont-medium">Ratings:</span>{" "}
							<span>4.5</span>
						</p>
						<p>
							<span className="font-medium">Volume:</span>{" "}
							<span>1K STX</span>
						</p>
					</div>
				</div>
				<Image
					src={game.imageUrl}
					alt="game logo"
					width={358}
					height={182}
					className="lg:w-89.5 w-24"
					loading="lazy"
				/>
			</div>
			<Button
				className="-translate-y-1/2 w-full lg:max-w-80 max-w-24 lg:px-12 px-3 lg:py-6 py-1.5 rounded-full lg:text-xl text-[5px] font-medium"
				asChild
			>
				<Link href={`/games/${game.path}`}>Play Now</Link>
			</Button>
		</div>
	);
}
