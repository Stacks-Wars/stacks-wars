import {
	Card,
	CardContent,
	CardHeader,
	CardTitle,
	CardFooter,
} from "@/components/ui/card";
import { GameType } from "@/types/schema/game";
import Image from "next/image";
//import { Button } from "@/components/ui/button";
//import { ChevronRight } from "lucide-react";

export default function GamePreview({ game }: { game: GameType }) {
	return (
		<Card className="bg-primary/10 overflow-hidden">
			<CardHeader className="p-4 pb-2 sm:p-6 sm:pb-3">
				<CardTitle className="text-sm sm:text-base">
					Game Preview
				</CardTitle>
			</CardHeader>
			<CardContent className="p-0">
				<Image
					src={game.imageUrl}
					width={500}
					height={300}
					alt="Game preview"
					className="h-auto w-full object-cover"
					priority={false}
				/>
			</CardContent>
			<CardFooter className="bg-muted/30 flex justify-between p-3 sm:p-4">
				<p className="text-xs font-medium sm:text-sm">{game.name}</p>
				{/*<Button
					variant="ghost"
					size="sm"
					className="h-7 sm:h-8 text-xs sm:text-sm gap-1 px-2 sm:px-3"
				>
					Game details
					<ChevronRight className="h-3 w-3 sm:h-4 sm:w-4" />
				</Button>*/}
			</CardFooter>
		</Card>
	);
}
