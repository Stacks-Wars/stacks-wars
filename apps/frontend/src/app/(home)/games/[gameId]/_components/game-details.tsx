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
					<div className="h-12 w-12 rounded-full bg-primary/10 flex items-center justify-center">
						<Gamepad2 className="h-6 w-6 text-primary" />
					</div>
					<div>
						<CardTitle className="text-2xl">{game.name}</CardTitle>
						<div className="flex gap-2 mt-2">
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
					<Trophy className="h-4 w-4 text-primary" />
					<span className="text-sm text-muted-foreground">
						Total Prize Pool:
					</span>
				</div>
			</CardContent>
		</Card>
	);
}
