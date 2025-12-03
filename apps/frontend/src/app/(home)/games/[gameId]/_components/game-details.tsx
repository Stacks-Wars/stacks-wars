import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Gamepad2, Trophy } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { GameType } from "@/types/schema/game";

export default function GameDetails({ game }: { game: GameType | null }) {
	if (!game) return null;
	return (
		<Card className="bg-primary/30">
			<CardHeader>
				<div className="flex items-center gap-3">
					<div className="bg-primary/10 flex h-12 w-12 items-center justify-center rounded-full">
						<Gamepad2 className="text-primary h-6 w-6" />
					</div>
					<div>
						<CardTitle className="text-2xl">{game.name}</CardTitle>
						<div className="mt-2 flex gap-2">
							{game.tags &&
								game.tags.map((tag) => (
									<Badge key={tag} variant="secondary">
										{tag}
									</Badge>
								))}
						</div>
					</div>
				</div>
			</CardHeader>
			<CardContent>
				<p className="text-muted-foreground mb-4">{game.description}</p>
				<div className="flex items-center gap-2">
					<Trophy className="text-primary h-4 w-4" />
					<span className="text-muted-foreground text-sm">
						Total Prize Pool:
					</span>
				</div>
			</CardContent>
		</Card>
	);
}
