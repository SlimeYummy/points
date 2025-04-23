using CriticalPoint;

namespace CriticalPointTests {
    [TestClass]
    public class TestEngine {
        const string TMPL_PATH = "../../../../../turning-point/test-templates";
        const string ASSET_PATH = "../../../../../turning-point/test-assets";

        [TestMethod]
        public void TestNewDelete() {
            Assert.ThrowsException<EngineException>(() => {
                LogicEngine engine = new LogicEngine("./test-templates", "./test-assets");
            });

            LogicEngine engine = new LogicEngine(TMPL_PATH, ASSET_PATH);
            Assert.IsNotNull(engine);
            engine.Dispose();
        }

        [TestMethod]
        public void TestVerifyPlayer() {
            using (var engine = new LogicEngine(TMPL_PATH, ASSET_PATH)) {
                var player1 = NewParamPlayer();
                Assert.AreEqual("OK", engine.VerifyPlayer(player1));

                var player2 = NewParamPlayer();
                player2.level = 10;
                Assert.AreNotEqual("OK", engine.VerifyPlayer(player2));

                var player3 = NewParamPlayer();
                player3.equipments = new List<IDLevel> {
                    new IDLevel { id = "Equipment.No1", level = 1 },
                    new IDLevel { id = "Equipment.No3", level = 0 },
                };
                Assert.AreNotEqual("OK", engine.VerifyPlayer(player3));
            }
        }

        ParamPlayer NewParamPlayer() {
            return new ParamPlayer {
                character = "Character.No1",
                style = "Style.No1-1",
                level = 6,
                equipments = new List<IDLevel> {
                    new IDLevel { id = "Equipment.No1", level = 4 },
                    new IDLevel { id = "Equipment.No2", level = 3 },
                },
                perks = new List<string> {
                    "Perk.No1.AttackUp",
                    "Perk.No1.Slot",
                },
                accessories = new List<ParamAccessory> {
                    new ParamAccessory { id = "Accessory.AttackUp.Variant1", level = 0, entries = new List<string> {"Entry.DefenseUp"} },
                },
                jewels = new List<IDPlus> {
                    new IDPlus { id = "Jewel.AttackUp.Variant1", plus = 0 },
                },
            };
        }

        ParamStage NewParamStage() {
            return new ParamStage {
                stage = "Stage.Demo",
            };
        }

        [TestMethod]
        public void TestStartGame() {
            using (var engine = new LogicEngine(TMPL_PATH, ASSET_PATH)) {
                var stage = NewParamStage();
                var player = NewParamPlayer();

                var state_set = engine.StartGame(stage, new List<ParamPlayer> { player });
                Assert.AreEqual(state_set.inits.Length, 3);
                state_set.Dispose();

                var state_sets = engine.UpdateGame(new List<PlayerEvents> {
                    new PlayerEvents {
                        frame = 1,
                        player_id = 100,
                        events = new List<KeyEvent> {
                             new KeyEvent { key = RawKey.Attack1, pressed = true },
                        },
                    }
                });
                state_sets.Dispose();

                engine.StopGame();
            }
        }
    }
}
