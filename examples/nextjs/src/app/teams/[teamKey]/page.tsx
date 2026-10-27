import { redirect } from "next/navigation";

export default function TeamPage({ params }: { params: { teamKey: string } }) {
  redirect(`/teams/${params.teamKey}/issues`);
}
