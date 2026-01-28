using CriticalPoint;

namespace CriticalPointTests {
    [TestClass]
    public class TestEngine {
        const string TMPL_PATH = "../../../../../test-tmp/test-template";
        const string ASSET_PATH = "../../../../../test-tmp/test-asset";
        const string LOG_PATH = "../../../../../test-tmp/test_cs.log";

        [AssemblyInitialize]
        public static void AssemblyInit(TestContext context) {
            LogicEngine.Initialize(TMPL_PATH, ASSET_PATH, LOG_PATH, LogicEngine.LOG_DEBUG);
        }

        [TestMethod]
        public void TestNewDelete() {
            LogicEngine engine = new LogicEngine();
            Assert.IsNotNull(engine);
            engine.Dispose();
        }

        [TestMethod]
        public void TestVerifyPlayer() {
            using (var engine = new LogicEngine()) {
                var player1 = NewParamPlayer();
                Assert.AreEqual("OK", engine.VerifyPlayer(player1));

                var player2 = NewParamPlayer();
                player2.level = 10;
                Assert.AreNotEqual("OK", engine.VerifyPlayer(player2));

                var player3 = NewParamPlayer();
                player3.equipments = new List<TmplIDLevel> {
                    new TmplIDLevel { id = new TmplID("Equipment.No1"), level = 1 },
                    new TmplIDLevel { id = new TmplID("Equipment.No3"), level = 0 },
                };
                Assert.AreNotEqual("OK", engine.VerifyPlayer(player3));
            }
        }

        ParamPlayer NewParamPlayer() {
            return new ParamPlayer {
                character = new TmplID("Character.One"),
                style = new TmplID("Style.One^1"),
                level = 6,
                equipments = new List<TmplIDLevel> {
                    new TmplIDLevel { id = new TmplID("Equipment.No1"), level = 4 },
                    new TmplIDLevel { id = new TmplID("Equipment.No2"), level = 3 },
                },
                perks = new List<TmplIDLevel> {
                    new TmplIDLevel { id = new TmplID("Perk.One.AttackUp"), level = 1 },
                    new TmplIDLevel { id = new TmplID("Perk.One.NormalAttack.Branch"), level = 1 },
                },
                accessories = new List<ParamAccessory> {
                    new ParamAccessory {
                        id = new TmplID("Accessory.AttackUp^1"), level = 0,
                        entries = new List<TmplID> { new TmplID("Entry.DefenseUp")},
                    },
                },
                jewels = new List<TmplIDPlus> {
                    new TmplIDPlus { id = new TmplID("Jewel.DefenseUp^1"), plus = 0 },
                },
            };
        }

        ParamZone NewParamZone() {
            return new ParamZone {
                zone = new TmplID("Zone.Demo"),
            };
        }

        [TestMethod]
        public void TestStartGame() {
            using (var engine = new LogicEngine()) {
                var zone = NewParamZone();
                var player = NewParamPlayer();

                var state_set = engine.StartGame(zone, new List<ParamPlayer> { player });
                Assert.AreEqual(state_set.inits.Length, 3);
                state_set.Dispose();

                var state_sets = engine.UpdateGame(new List<InputPlayerInputs> {
                    new InputPlayerInputs {
                        frame = 1,
                        player_id = 100,
                        inputs = new List<RawInput> {
                             new RawInput { key = RawKey.Attack1, pressed = true },
                        },
                    }
                });
                state_sets.Dispose();

                engine.StopGame();
            }
        }
    }
}
