import { useVerge } from "@/hooks/use-verge";
import {
  Button,
  Checkbox,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  FormControlLabel,
  FormGroup,
} from "@mui/material";
import { red } from "@mui/material/colors";
import { useState } from "react";
import { useTranslation } from "react-i18next";

const LazyTestCard = lazy(() =>
  import("@/components/home/test-card").then((module) => ({
    default: module.TestCard,
  })),
);

const LazyIpInfoCard = lazy(() =>
  import("@/components/home/ip-info-card").then((module) => ({
    default: module.IpInfoCard,
  })),
);

const LazyClashInfoCard = lazy(() =>
  import("@/components/home/clash-info-card").then((module) => ({
    default: module.ClashInfoCard,
  })),
);

const LazySystemInfoCard = lazy(() =>
  import("@/components/home/system-info-card").then((module) => ({
    default: module.SystemInfoCard,
  })),
);

// 定义首页卡片设置接口
interface HomeCardsSettings {
  profile: boolean;
  proxy: boolean;
  network: boolean;
  mode: boolean;
  traffic: boolean;
  info: boolean;
  clashinfo: boolean;
  systeminfo: boolean;
  ip: boolean;
  [key: string]: boolean;
  test: boolean;
}

interface HomeSettingsDialogProps {
  open: boolean;
  onClose: () => void;
  homeCards: HomeCardsSettings;
  onSave: (cards: HomeCardsSettings) => void;
}
const serializeCardFlags = (cards: HomeCardsSettings) =>
  Object.keys(cards)
    .sort()
    .map((key) => `${key}:${cards[key] ? 1 : 0}`)
    .join("|");

const HomeSettingsDialog: React.FC<HomeSettingsDialogProps> = ({
  open,
  onClose,
  homeCards,
  onSave,
}: HomeSettingsDialogProps) => {
  const { t } = useTranslation();
  const [cards, setCards] = useState<HomeCardsSettings>(homeCards);
  const { patchVerge } = useVerge();

  const handleToggle = (key: string) => {
    setCards((prev: HomeCardsSettings) => ({
      ...prev,
      [key]: !prev[key],
    }));
  };

  const handleSave = async () => {
    onSave(cards);
    await patchVerge({
      home_cards: cards,
    });
    onClose();
  };

  return (
    <Dialog open={open} onClose={onClose} maxWidth="xs" fullWidth>
      <DialogTitle>{t("home.page.settings.title")}</DialogTitle>
      <DialogContent>
        <FormGroup>
          <FormControlLabel
            control={
              <Checkbox
                checked={cards.profile || false}
                onChange={() => handleToggle("profile")}
              />
            }
            label={t("home.page.settings.cards.profile")}
          />
          <FormControlLabel
            control={
              <Checkbox
                checked={cards.proxy || false}
                onChange={() => handleToggle("proxy")}
              />
            }
            label={t("home.page.settings.cards.currentProxy")}
          />
          <FormControlLabel
            control={
              <Checkbox
                checked={cards.network || false}
                onChange={() => handleToggle("network")}
              />
            }
            label={t("home.page.settings.cards.network")}
          />
          <FormControlLabel
            control={
              <Checkbox
                checked={cards.mode || false}
                onChange={() => handleToggle("mode")}
              />
            }
            label={t("home.page.settings.cards.mode")}
          />
          <FormControlLabel
            control={
              <Checkbox
                checked={cards.traffic || false}
                onChange={() => handleToggle("traffic")}
              />
            }
            label={t("home.page.settings.cards.traffic")}
          />
          // TODO: Add more cards here
        </FormGroup>
      </DialogContent>
      <DialogActions>
        <Button onClick={onClose}>{t("shared.actions.cancel")}</Button>
        <Button onClick={handleSave} color="primary">
          {t("shared.actions.save")}
        </Button>
      </DialogActions>
    </Dialog>
  );
};

const HomePage = () => {
  const { t } = useTranslation();
  const { verge } = useVerge();
  const [current, muateProfiles] = useProfiles();
  return (
    <div>
      <h1>Home Page</h1>
      <div style={{ backgroundColor: red[500], height: "100px" }}></div>
    </div>
  );
};

export default HomePage;
