import { Button } from "@/components/ui/button";
import {
	Card,
	CardHeader,
	CardTitle,
	CardDescription,
	CardContent,
	CardFooter,
} from "@/components/ui/card";
import {
	//Trophy,
	ArrowRight,
} from "lucide-react";
import Link from "next/link";
import Image from "next/image";
import { Badge } from "@/components/ui/badge";
import { GameType } from "@/types/schema/game";

export default function GameCard({ game }: { game: GameType }) {
	return (
		<Card className="bg-primary/30">
			<div className="sm:grid sm:grid-cols-[1fr_300px]">
				<div>
					<CardHeader className="flex items-start justify-between">
						<CardTitle className="text-2xl">{game.name}</CardTitle>
						<CardDescription className="mt-1">
							{game.tags &&
								game.tags.map((tag) => (
									<Badge
										variant="outline"
										key={tag}
										className="mr-2"
									>
										{tag}
									</Badge>
								))}
						</CardDescription>
					</CardHeader>
					<CardContent>
						<p className="mb-4">{game.description}</p>

						{/*<div className="flex items-center gap-2">
							<Trophy className="h-5 w-5 text-muted-foreground" />
							<div>
								<p className="text-sm font-medium">
									Total Volume
								</p>
								<p className="text-lg font-bold">
									{game.totalPrize} STX
								</p>
							</div>
						</div>*/}
					</CardContent>
					<CardFooter className="mt-3">
						<div className="flex gap-4">
							{/*<Link href={`/lobby?game=${game.id}`}>
								<Button variant="outline">Learn More</Button>
							</Link>*/}
							<Button asChild>
								<Link href={`/games/${game.id}`}>
									Enter Game
									<ArrowRight />
								</Link>
							</Button>
						</div>
					</CardFooter>
				</div>
				<div className="relative mr-6 hidden sm:block">
					<Image
						src={game.imageUrl}
						alt={game.name}
						className="absolute inset-0 h-full w-full object-cover"
						width={300}
						height={300}
						loading="lazy"
					/>
				</div>
				<div className="mt-4 sm:hidden">
					<Image
						src={game.imageUrl}
						alt={game.name}
						className="w-full rounded-md object-cover"
						width={400}
						height={200}
						loading="lazy"
					/>
				</div>
			</div>
		</Card>
	);
}
