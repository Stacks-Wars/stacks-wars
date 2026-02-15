import { siteConfig } from "@stacks-wars/shared";
import Image from "next/image";
import { Button } from "@/components/ui/button";
import { ExternalLink } from "../ui/external-link";

export default function Footer() {
	return (
		<footer className="mt-16 sm:mt-20 lg:mt-28">
			<div className="container mx-auto px-4 py-8 sm:py-12 lg:py-16">
				<Image
					src={"/images/footer-seperator.svg"}
					alt="Footer Illustration"
					width={1248}
					height={28}
					className="mb-6 h-4 w-full object-cover sm:mb-8 sm:h-7 lg:mb-12"
				/>
				<div className="mx-auto max-w-4xl space-y-6 sm:space-y-8 lg:space-y-12">
					<div className="space-y-4 sm:space-y-6">
						<p className="text-foreground text-center text-2xl font-bold sm:text-3xl lg:text-4xl xl:text-5xl">
							Every stack counts. Every move matters. Are you ready?
						</p>
						<p className="text-foreground text-center text-lg font-medium sm:text-xl lg:text-2xl xl:text-3xl">
							Join the community
						</p>
					</div>
					<div className="flex flex-col items-center justify-center gap-4 sm:flex-row sm:gap-6">
						<Button
							asChild
							variant="outline"
							size="lg"
							className="w-full max-w-xs cursor-pointer rounded-full py-3 text-sm font-medium sm:w-auto sm:max-w-none sm:px-8 sm:py-4 sm:text-base lg:px-10 lg:py-5 lg:text-lg"
						>
							<ExternalLink href={siteConfig.socials.x}>
								X (formerly Twitter)
							</ExternalLink>
						</Button>
						<Button
							asChild
							variant="outline"
							size="lg"
							className="w-full max-w-xs cursor-pointer rounded-full py-3 text-sm font-medium sm:w-auto sm:max-w-none sm:px-8 sm:py-4 sm:text-base lg:px-10 lg:py-5 lg:text-lg"
						>
							<ExternalLink href={siteConfig.socials.telegram}>
								Telegram
							</ExternalLink>
						</Button>
					</div>
				</div>
			</div>

			<Image
				src={"/images/stacks-wars.svg"}
				alt="Footer Text"
				width={1920}
				height={120}
				className="h-auto w-full object-cover"
			/>
		</footer>
	);
}
