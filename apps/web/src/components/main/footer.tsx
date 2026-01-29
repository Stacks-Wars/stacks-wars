"use client";

import Image from "next/image";
import { Button } from "@/components/ui/button";

const FooterImg = "/images/footer-img.svg";
const FooterText = "/images/footer-text.svg";

export default function Footer() {
	return (
		<footer className="mt-16 w-full py-8 md:mt-32">
			<div className="container mx-auto flex w-full flex-col items-center justify-between gap-2 md:gap-8">
				<div className="mb-12 w-full md:mb-38">
					<Image
						src={FooterImg}
						alt="Footer Illustration"
						width={1920}
						height={120}
						className="h-12 w-full object-cover md:h-auto"
						priority
					/>
				</div>
				<h2 className="text-foreground mb-2 text-center text-3xl font-bold md:mb-4 md:text-5xl">
					Every stack counts. Every move matters. Are you ready?
				</h2>
				<p className="text-foreground text-center text-2xl font-medium md:text-3xl">
					Join the community
				</p>
				<div className="flex flex-row items-center justify-center gap-4 md:w-auto md:flex-row">
					<Button
						asChild
						variant="outline"
						size="lg"
						className="cursor-pointer rounded-full p-6"
					>
						<a
							href="https://x.com/stacks-wars"
							target="_blank"
							rel="noopener noreferrer"
						>
							X (formerly Twitter)
						</a>
					</Button>
					<Button
						asChild
						variant="outline"
						size="lg"
						className="cursor-pointer rounded-full p-6"
					>
						<a
							href="https://t.me/stacks-wars"
							target="_blank"
							rel="noopener noreferrer"
						>
							Telegram
						</a>
					</Button>
				</div>

				<div className="w-full">
					<Image
						src={FooterText}
						alt="Footer Text"
						width={1920}
						height={120}
						className="bottom-0 mt-28 hidden h-auto w-full object-cover md:block"
						priority
					/>
				</div>
			</div>
		</footer>
	);
}
