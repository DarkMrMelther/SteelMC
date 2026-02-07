use steel_macros::{ClientPacket, WriteTo};
use steel_registry::packets::play::C_ENTITY_EVENT;

/// Status type for the entity event packet.
#[derive(WriteTo, Clone, Copy, Debug, PartialEq, Eq)]
#[write(as = VarInt)]
pub enum EntityStatus {
    /// Applicable to: Arrow
    TippedParticles = 0,
    /// In case of the Minecart Spawner it resets the spawn delay to 200\
    /// Applicable to: Minecart Spawner, Rabbit
    JumpAnimation = 1,
    /// Applicable to: None
    C = 2,
    /// Applicable to: Egg, Living Entity, Snowball
    DeathAnimation = 3,
    /// Applicable to: Evoker Fangs, Hoglin, Iron Golem, Ravager, Warden, Zoglin
    AttackAnimation = 4,
    /// Applicable to: None
    F = 5,
    /// Applicable to: Abstract Horse, Tameable Animal
    TameFailParticles = 6,
    /// Applicable to: Abstract Horse, Tameable Animal
    TameSuccessParticles = 7,
    /// Applicable to: Wolf
    StartWolfShakingAnimation = 8,
    /// Applicable to: Player
    ItemUseFinished = 9,
    /// Applicable to: Minecart TNT, Sheep
    EatGrassOrIgnite = 10,
    /// Applicable to: Iron Golem
    StartHoldingPoppy = 11,
    /// Applicable to: Villager
    VillagerBreedParticles = 12,
    /// Applicable to: Villager
    VillagerAngryParticles = 13,
    /// Applicable to: Villager
    VillagerHappyParticles = 14,
    /// Applicable to: Witch
    WitchMagicParticles = 15,
    /// Applicable to: Zombie Villager
    VillagerCureSound = 16,
    /// Applicable to: Firework Rocket
    ExplosionEffect = 17,
    /// Applicable to: Allay, Animal
    BreedParticles = 18,
    /// Applicable to: Squid
    ResetRotation = 19,
    /// Applicable to: Mob
    SpawnParticles = 20,
    /// Applicable to: Guardian
    GuardianAttackSound = 21,
    /// Applicable to: Player
    ReducedDebugEnabled = 22,
    /// Applicable to: Player
    ReducedDebugDisabled = 23,
    /// Applicable to: Player
    OpLevel0 = 24,
    /// Applicable to: Player
    OpLevel1 = 25,
    /// Applicable to: Player
    OpLevel2 = 26,
    /// Applicable to: Player
    OpLevel3 = 27,
    /// Applicable to: Player
    OpLevel4 = 28,
    /// Applicable to: Living Entity
    ShieldBlockSound = 29,
    /// Applicable to: Living Entity
    ShieldBreakSound = 30,
    /// Applicable to: Fishing Hook
    PullPlayer = 31,
    /// Applicable to: Armor Stand
    Hit = 32,
    /// Applicable to: None
    AH = 33,
    /// Applicable to: Iron Golem
    StopHoldingPoppy = 34,
    /// Applicable to: Living Entity
    TotemAnimation = 35,
    /// Applicable to: None
    AK = 36,
    /// Applicable to: None
    AL = 37,
    /// Applicable to: Dolphin
    DolphinFedParticles = 38,
    /// Applicable to: Ravager
    MarkAsStunned = 39,
    /// Applicable to: Ocelot
    OcelotTameFailParticles = 40,
    /// Applicable to: Ocelot
    OcelotTameSuccessParticles = 41,
    /// Applicable to: Villager
    VillagerWorriedParticles = 42,
    /// Applicable to: Player
    BadOmenCloudParticles = 43,
    /// Applicable to: None
    AS = 44,
    /// Applicable to: Fox
    ChewItemParticles = 45,
    /// Applicable to: Living Entity
    TeleportParticles = 46,
    /// Applicable to: Living Entity
    MainHandItemBreak = 47,
    /// Applicable to: Living Entity
    OffHandItemBreak = 48,
    /// Applicable to: Living Entity
    HeadItemBreak = 49,
    /// Applicable to: Living Entity
    ChestItemBreak = 50,
    /// Applicable to: Living Entity
    LegsItemBreak = 51,
    /// Applicable to: Living Entity
    FeetItemBreak = 52,
    /// Applicable to: Entity
    HoneySlideParticles = 53,
    /// Applicable to: Living Entity
    HoneyFallParticles = 54,
    /// Applicable to: Living Entity
    InterchangeHandItems = 55,
    /// Applicable to: Wolf
    StopWolfShakingAnimation = 56,
    /// Applicable to: None
    BF = 57,
    /// Applicable to: Goat
    HeadDown = 58,
    /// Applicable to: Goat
    HeadUp = 59,
    /// Applicable to: Living Entity
    DeathSmokeParticles = 60,
    /// Applicable to: Warden
    TendrilShakingAnimation = 61,
    /// Applicable to: Warden
    SonicBoomAnimation = 62,
    /// Applicable to: Sniffer
    DiggingSound = 63,
}

/// Performs an entity event.
#[derive(ClientPacket, WriteTo, Clone, Debug)]
#[packet_id(Play = C_ENTITY_EVENT)]
pub struct CEntityEvent {
    pub entity_id: i32,
    pub event: EntityStatus,
}
